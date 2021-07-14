#!/usr/bin/env bash

# constants
RED='\033[0;31m'
NC='\033[0m'

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
if [ $? -eq 0 ]; then
  list="$(pacman -Qq | xargs pacman -Qi | grep URL | grep 'https://github.com/' | sed 's/  */|/g' | cut -d '|' -f 3 | sed -E 's_https://github.com/([-_\.a-zA-Z0-9]+)/([-_\.a-zA-Z0-9]+).*_\1/\2_' | sed '/:/d' | sort | uniq) $list"
fi

# star!
for repo in $list; do
  echo -n "Starring github.com/$repo ..."
  echo $repo | sed "s_^_/user/starred/_" | xargs echo -n
  if [[ $? -ne 0 ]]; then
    echo " ${RED}error!${NC}"
  else
    echo " done"
  fi
done
