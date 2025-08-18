#!/usr/bin/env bash
source settings.inc

SM83_JSON="${TOOLS}/sm83/v1"

for i in $( seq 0 255 ); do
  index=$(printf "%02x" $i)
  file="${SM83_JSON}/${index}".json

  if [ ! -f "$file" ]; then
    continue
  fi
  
  cargo run --release --bin sm83-doctor -- "${file}"

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