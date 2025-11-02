#!/usr/bin/env bash

source settings.inc
mkdir -p "${LOGS}"

git -C "${GAMEBOY_DOCTOR_DIR}" pull || git clone https://github.com/robert/gameboy-doctor --depth 1 "${GAMEBOY_DOCTOR_DIR}"

mkdir -p "${ROMS}"
if ! [ -f "${ROMS}/.ok" ]; then
  curl -LO https://github.com/c-sp/game-boy-test-roms/releases/download/v7.0/game-boy-test-roms-v7.0.zip
  unzip -n game-boy-test-roms-v7.0.zip -d "${ROMS}"
  rm game-boy-test-roms-v7.0.zip
  touch "${ROMS}/.ok"
fi

git -C "${GAMEBOY_SM83_DIR}" pull || git clone https://github.com/SingleStepTests/sm83 --depth 1 "${GAMEBOY_SM83_DIR}"

## -------- Demos ------------ ##

mkdir -p "${DEMOS}"
cd "${DEMOS}" || exit

function download() {
  if ! [ -f "$1" ]; then
    curl -LO "$2"
  fi
}

download cncd-at.zip https://files.scene.org/get/parties/2002/altparty02/demo/cncd-at.zip
unzip -n cncd-at.zip
download 8-bit_beauty.zip https://files.scene.org/get/parties/2025/revision25/oldskool-demo/8-bit_beauty.zip

cd - || exit