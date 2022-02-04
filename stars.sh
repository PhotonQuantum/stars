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
YUM=0
GOLANG=0
GENTOO=0

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
    pacman|archlinux|manjaro)
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
      ;;
    yum|centos|fedora)
      echo "Argument $1: yum module enabled"
      TARGET=1
      YUM=1
      shift
      ;;
    gentoo)
      echo "Argument $1: gentoo module enabled"
      TARGET=1
      GENTOO=1
      shift
      ;;
    golang|go)
      echo "Argument $1: golang module enabled"
      TARGET=1
      GOLANG=1
      shift
      ;;
    *)
      echo "Unknown argument: $1"
      exit 1
      ;;
  esac
done

# sanity check
function check() {
  for com in $1; do
    which $com &> /dev/null
    if [ $? -ne 0 ]; then
      echo -e "${RED}$com not found in PATH${NC}"
      exit 1
    fi
  done
  return 0
}

function exist() {
  for file in $1; do
    if [[ ! -f "$file" ]]; then
      echo -e "${RED}$1 not found in current directory${NC}"
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

## Arch Linux/Manjaro etc
if [[ $PACMAN -eq 1 ]]; then
  check "pacman cut"
  list="$(pacman -Qi | grep URL | grep 'https://github.com/' | sed 's/  */|/g' | cut -d '|' -f 3 | extract) $list"
fi

## Debian/Ubuntu etc
if [[ $DPKG -eq 1 ]]; then
  check "dpkg-query"
  list="$(dpkg-query -f '${Homepage}\n' -W | grep 'https://github.com/' | extract) $list"
fi

## CentOS/Fedora etc
if [[ $YUM -eq 1 ]]; then
  check "yum"
  list="$(yum info installed 2> /dev/null | grep URL | grep 'https://github.com/' | sed 's/  */|/g' | cut -d '|' -f 3 | extract) $list"
fi

## Gentoo
if [[ $GENTOO -eq 1 ]]; then
  check "equery"
  list="$(equery list '*' | xargs equery meta | grep Homepage | grep 'https://github.com/' | sed 's/  */|/g' | cut -d '|' -f 2 | extract) $list"
fi

## Golang
if [[ $GOLANG -eq 1 ]]; then
  exist "go.sum"
  list="$(cat go.sum | grep 'github.com' | cut -d ' ' -f 1 | sed 's_^_https://_' | extract) $list"
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
