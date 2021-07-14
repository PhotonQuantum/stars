#!/usr/bin/env bash

# constants
RED='\033[0;31m'
NC='\033[0m'

# parse arguments
DRYRUN=0
QUIET=0
BURST=0

TARGET=0
PACMAN=0
DPKG=0

while [[ $# -gt 0 ]]; do
  case "$1" in
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
    pacman|archlinux)
      echo "Argument $1: pacman module enabled"
      TARGET=1
      PACMAN=1
      shift
      ;;
    dpkg|ubuntu|debian)
      echo "Argument $1: dpkg module enabled"
      TARGET=1
      DPKG=1
      shift
  esac
done

# sanity check
function check() {
  for com in $1; do
    which $com &> /dev/null
    if [ $? -ne 0 ]; then
      echo -e "${RED}$com not found${NC}"
      exit 1
    fi
  done
  return 0
}

# generate list
list=""
if [[ $DRYRUN -eq 0 ]]; then
  check "gh"
fi

check "grep sed sort uniq xargs"

function extract() {
  sed -E 's_https://github.com/([-_\.a-zA-Z0-9]+)/([-_\.a-zA-Z0-9]+).*_\1/\2_' | sed '/:/d' | sort | uniq
}

## Arch Linux
if [[ $PACMAN -eq 1 ]]; then
  check "pacman cut"
  list="$(pacman -Qq | xargs pacman -Qi | grep URL | grep 'https://github.com/' | sed 's/  */|/g' | cut -d '|' -f 3 | extract) $list"
fi

## Debian/Ubuntu etc
if [[ $DPKG -eq 1 ]]; then
  check "dpkg-query"
  list="$(dpkg-query -f '${Homepage}\n' -W | grep 'https://github.com/' | extract) $list"
fi

## No target
if [[ $TARGET -eq 0 ]]; then
  echo "No target(e.g. pacman, dpkg) specified"
  exit 1
fi

# star!
for repo in $list; do
  com="gh api --silent -X PUT"
  if [[ $DRYRUN -eq 1 ]]; then
    echo $repo | sed "s_^_/user/starred/_" | xargs -I {} echo "Will run: gh api --silent -X PUT" {}
  else
    echo $repo | sed "s_^_/user/starred/_" | xargs gh api --silent -X PUT > /dev/null
  fi
  if [[ $? -ne 0 ]]; then
    echo -e "${RED}! Error when star github.com/$repo${NC}"
  elif [[ $QUIET -ne 1 ]]; then
    echo -e "Starred ${repo}"
  fi
  if [[ $BURST -ne 1 ]]; then
    sleep 1 # sleep due to rate limit
  fi
done
