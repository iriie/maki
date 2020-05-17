#!/bin/bash
git log -1 | grep ^commit | cut -d " " -f 2