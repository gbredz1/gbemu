#!/usr/bin/env bash
source settings.inc

BLARGG_ROM="${ROMS}/blargg"

function run-test() {
  RUST_LOG=off cargo run -q --release --bin blargg-doctor -- $1
  if [ $? -ne 0 ]; then
    RESULT="${RESULT} $1"
    if [ $EXIT_ON_FIRST_FAILED -ne 0 ]; then
      exit
    fi
  fi
  echo ""
}

echo ""
echo "----- cpu_instrs -----"
echo ""
for i in $( seq 1 11 ); do
  index=$(printf "%02d" $i)

  run-test "${BLARGG_ROM}/cpu_instrs/individual/${index}-*.gb"
done

echo ""
echo "----- instr_timing -----"
echo ""
run-test "${BLARGG_ROM}/instr_timing/instr_timing.gb"

echo ""
echo "----- mem_timing -----"
echo ""
run-test "${BLARGG_ROM}/mem_timing/mem_timing.gb"

echo ""
echo "----------"
echo ""

if [[ -n $RESULT ]]; then
  echo "FAILED: $RESULT"
  exit 1
fi

echo "SUCCESS!!"
exit 0