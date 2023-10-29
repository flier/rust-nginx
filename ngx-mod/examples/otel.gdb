# http://sourceware.org/gdb/wiki/FAQ: to disable the
# "---Type <return> to continue, or q <return> to quit---"
# in batch mode:
set width 0
set height 0
set verbose off
set pagination off
set breakpoint pending on
set follow-fork-mode child

show args

br set_exporter
br parse_block

r
