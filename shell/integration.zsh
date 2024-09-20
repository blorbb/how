# https://github.com/junegunn/fzf/blob/master/shell/key-bindings.zsh
# fzf uses:
# echo -n -E "${(q)item} "
# which does a safe quotation of $item.
# outputting a command, so we don't want it to be quoted.

how-select() {
  setopt localoptions pipefail no_aliases 2> /dev/null
  local item
  how "$@" < /dev/tty | while read -r item; do
    echo -n -E "${item}"
  done
  local ret=$?
  echo
  return $ret
}

how-widget() {
  LBUFFER="${LBUFFER}$(how-select)"
  local ret=$?
  zle reset-prompt
  return $ret
}

zle -N how-widget
bindkey '^_' how-widget