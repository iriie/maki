#!/bin/bash
pmap $1 | head -n 2 | tail -n 1 | awk '/[0-9]K/{print $2/1000}'