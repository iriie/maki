# maki (discord bot)
wip

bad bad code, do not use this as any sort of inspiration

why did i do this

ugh

## how to use
make sure you have an env file in your root project directory that looks like this, obviously with everything filled out:
```
PREFIX=
BOT_TOKEN=
BING_MAPS_KEY=
DARK_SKY_KEY=
```

run on your own system:

`> git clone && cargo run`

or to run with docker:
```
> git clone
> docker build -t maki-bot .
> docker run maki-bot --env-file ./.env
```
## other info
tested on windows 10 and linux, may or may not work on macos
