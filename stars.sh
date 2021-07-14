#!/usr/bin/env bash

# constants
RED='\033[0;31m'
NC='\033[0m'

# parse arguments
DRYRUN=0
QUIET=0
BURST=0

while [[ $# -gt 0 ]]; do
  key="$1"
  case $key in
    -d|--dryrun)
      DRYRUN=1
      shift
      ;;
    -q|--quiet)
      QUIET=1
      shift
      ;;
    -b|--burst)
      echo "You have been warned!"
      BURST=1
      shift
      ;;
  esac
done

# sanity check
function check() {
  for com in $1; do
    which $com > /dev/null
    if [ $? -ne 0 ]; then
      return 1
    fi
  done
  return 0
}

# generate list

list=""

## Arch Linux
check "pacman xargs grep sed cut sort uniq head"
if [[ $? -eq 0 ]]; then
  list="$(pacman -Qq | xargs pacman -Qi | grep URL | grep 'https://github.com/' | sed 's/  */|/g' | cut -d '|' -f 3 | sed -E 's_https://github.com/([-_\.a-zA-Z0-9]+)/([-_\.a-zA-Z0-9]+).*_\1/\2_' | sed '/:/d' | sort | uniq) $list"
fi

# star!
for repo in $list; do
  com="gh"
  if [[ $DRYRUN -eq 1 ]]; then
    com="echo"
  fi
  echo $repo | sed "s_^_/user/starred/_" | xargs $com > /dev/null
  if [[ $? -ne 0 ]]; then
    echo -e "${RED}! Error when star github.com/$repo${NC}"
  elif [[ $QUIET -ne 1 ]]; then
    echo -e "Starred ${repo}"
  fi
  if [[ $BURST -ne 1 ]]; then
    sleep 1 # sleep due to rate limit
  fi
done
