// This file was taken from https://github.com/serenity-rs/songbird/blob/next/src/tracks/queue.rs.

use parking_lot::Mutex;
use serenity::async_trait;
use songbird::{
    //driver::Driver,
    events::{Event, EventContext, EventData, EventHandler, TrackEvent},
    input::{Input, Metadata},
    tracks::{self, Track, TrackHandle, TrackResult},
    Call,
};

use std::{collections::VecDeque, ops::Deref, sync::Arc};
use tokio::sync::MutexGuard;

/// A simple queue for several audio sources, designed to
/// play in sequence.
///
/// This makes use of [`TrackEvent`]s to determine when the current
/// song or audio file has finished before playing the next entry.
///

#[derive(Clone, Debug, Default)]
pub struct TrackQueue {
    // NOTE: the choice of a parking lot mutex is quite deliberate
    inner: Arc<Mutex<TrackQueueCore>>,
}

/// Reference to a track which is known to be part of a queue.
///
/// Instances *should not* be moved from one queue to another.
#[derive(Debug)]
pub struct Queued(TrackHandle, Metadata);

impl Deref for Queued {
    type Target = TrackHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Queued {
    /// Clones the inner handle
    pub fn handle(&self) -> TrackHandle {
        self.0.clone()
    }
}

#[derive(Debug, Default)]
/// Inner portion of a [`TrackQueue`].
///
/// This abstracts away thread-safety from the user,
/// and offers a convenient location to store further state if required.
///
/// [`TrackQueue`]: TrackQueue
struct TrackQueueCore {
    tracks: VecDeque<Queued>,
}

struct QueueHandler {
    remote_lock: Arc<Mutex<TrackQueueCore>>,
}

#[async_trait]
impl EventHandler for QueueHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let mut inner = self.remote_lock.lock();

        // Due to possibility that users might remove, reorder,
        // or dequeue+stop tracks, we need to verify that the FIRST
        // track is the one who has ended.
        let front_ended = match ctx {
            EventContext::Track(ts) => {
                // This slice should have exactly one entry.
                // If the ended track has same id as the queue head, then
                // we can progress the queue.
                let queue_uuid = inner.tracks.front().map(|handle| handle.uuid());
                let ended_uuid = ts.first().map(|handle| handle.1.uuid());

                queue_uuid.is_some() && queue_uuid == ended_uuid
            }
            _ => false,
        };

        if !front_ended {
            return None;
        }

        let _old = inner.tracks.pop_front();

        info!("Queued track ended: {:?}.", ctx);
        info!("{} tracks remain.", inner.tracks.len());

        // Keep going until we find one track which works, or we run out.
        let mut keep_looking = true;
        while keep_looking && !inner.tracks.is_empty() {
            if let Some(new) = inner.tracks.front() {
                keep_looking = new.play().is_err();

                // Discard files which cannot be used for whatever reason.
                if keep_looking {
                    warn!("Track in Queue couldn't be played...");
                    let _ = inner.tracks.pop_front();
                }
            }
        }

        None
    }
}

impl TrackQueue {
    /// Create a new, empty, track queue.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(TrackQueueCore {
                tracks: VecDeque::new(),
            })),
        }
    }

    /// Adds an audio source to the queue, to be played in the channel managed by `handler`.
    pub fn add_source(
        &self,
        source: Input,
        handler: &mut MutexGuard<Call>,
    ) {
        let meta = source.metadata.clone();
        let (audio, _) = tracks::create_player(source);
        self.add(audio, meta, handler);
    }

    /// Adds a [`Track`] object to the queue, to be played in the channel managed by `handler`.
    ///
    /// This is used with [`create_player`] if additional configuration or event handlers
    /// are required before enqueueing the audio track.
    ///
    /// [`Track`]: Track
    /// [`create_player`]: super::create_player
    pub fn add(
        &self,
        mut track: Track,
        metadata: Metadata,
        handler: &mut MutexGuard<Call>,
    ) {
        self.add_raw(&mut track, metadata);
        handler.play(track);
    }

    #[inline]
    pub(crate) fn add_raw(&self, track: &mut Track, metadata: Metadata) {
        info!("Track added to queue.");
        let remote_lock = self.inner.clone();
        let mut inner = self.inner.lock();

        let track_handle = track.handle.clone();

        if !inner.tracks.is_empty() {
            track.pause();
        }

        let pos = track.position().to_owned();

        track
            .events
            .as_mut()
            .expect("Queue inspecting EventStore on new Track: did not exist.")
            .add_event(
                EventData::new(Event::Track(TrackEvent::End), QueueHandler { remote_lock }),
                pos,
            );

        inner.tracks.push_back(Queued(track_handle, metadata));
    }

    /// Returns a handle to the currently playing track.
    pub fn current(&self) -> Option<TrackHandle> {
        let inner = self.inner.lock();

        inner.tracks.front().map(|h| h.handle())
    }

    /// Attempts to remove a track from the specified index.
    ///
    /// The returned entry can be readded to *this* queue via [`modify_queue`].
    ///
    /// [`modify_queue`]: TrackQueue::modify_queue
    pub fn dequeue(&self, index: usize) -> Option<Queued> {
        self.modify_queue(|vq| vq.remove(index))
    }

    /// Returns the number of tracks currently in the queue.
    pub fn len(&self) -> usize {
        let inner = self.inner.lock();

        inner.tracks.len()
    }

    /// Returns whether there are no tracks currently in the queue.
    pub fn is_empty(&self) -> bool {
        let inner = self.inner.lock();

        inner.tracks.is_empty()
    }

    /// Allows modification of the inner queue (i.e., deletion, reordering).
    ///
    /// Users must be careful to `stop` removed tracks, so as to prevent
    /// resource leaks.
    pub fn modify_queue<F, O>(&self, func: F) -> O
    where
        F: FnOnce(&mut VecDeque<Queued>) -> O,
    {
        let mut inner = self.inner.lock();
        func(&mut inner.tracks)
    }

    /// Pause the track at the head of the queue.
    pub fn pause(&self) -> TrackResult<()> {
        let inner = self.inner.lock();

        if let Some(handle) = inner.tracks.front() {
            handle.pause()
        } else {
            Ok(())
        }
    }

    /// Resume the track at the head of the queue.
    pub fn resume(&self) -> TrackResult<()> {
        let inner = self.inner.lock();

        if let Some(handle) = inner.tracks.front() {
            debug!("Resuming track.");
            handle.play()
        } else {
            Ok(())
        }
    }

    /// Stop the currently playing track, and clears the queue.
    pub fn stop(&self) {
        let mut inner = self.inner.lock();

        for track in inner.tracks.drain(..) {
            // Errors when removing tracks don't really make
            // a difference: an error just implies it's already gone.
            let _ = track.stop();
        }
    }

    /// Skip to the next track in the queue, if it exists.
    pub fn skip(&self) -> TrackResult<()> {
        let inner = self.inner.lock();

        inner.stop_current()
    }

    /// Returns a list of currently queued tracks.
    ///
    /// Does not allow for modification of the queue, instead returns a snapshot of the queue at the time of calling.
    ///
    /// Use [`modify_queue`] for direct modification of the queue.
    ///
    /// [`modify_queue`]: TrackQueue::modify_queue
    pub fn current_queue(&self) -> Vec<TrackHandle> {
        let inner = self.inner.lock();

        inner.tracks.iter().map(|q| q.handle()).collect()
    }
}

impl TrackQueueCore {
    /// Skip to the next track in the queue, if it exists.
    fn stop_current(&self) -> TrackResult<()> {
        if let Some(handle) = self.tracks.front() {
            handle.stop()
        } else {
            Ok(())
        }
    }
}
