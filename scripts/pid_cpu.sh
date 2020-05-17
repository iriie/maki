#!/bin/bash
top -b -n 2 -d 0.2 -p $1 | tail -1 | awk '{print $9}'