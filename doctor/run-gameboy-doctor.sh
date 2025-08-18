#!/usr/bin/env bash
source settings.inc

BLARGG_ROM="${ROMS}/blargg"

for i in $( seq 1 11 ); do
  index=$(printf "%02d" $i)

  file_gb="${BLARGG_ROM}/cpu_instrs/individual/${index}-*.gb"
  file_log="${LOGS}/doctor_${index}.log"

  cargo run --release --bin gameboy-doctor -- ${file_gb} > ${file_log}
  ${GAMEBOY_DOCTOR} ${file_log} cpu_instrs ${index}

  if [ $? -ne 0 ]; then
    RESULT="${RESULT} $index"

    if [ $EXIT_ON_FIRST_FAILED -ne 0 ]; then
      break
    fi

  fi
done

echo ""
echo "----------"
echo ""

if [[ -n $RESULT ]]; then
  echo "FAILED: $RESULT"
  exit 1
fi

echo "SUCCESS!!"
exit 0