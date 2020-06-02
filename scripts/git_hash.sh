#!/bin/bash
git log -1 | grep ^commit | awk '{print $2}'