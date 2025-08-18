#!/usr/bin/env bash

source settings.inc
mkdir -p "${LOGS}"

git -C "${GAMEBOY_DOCTOR_DIR}" pull || git clone https://github.com/robert/gameboy-doctor --depth 1 "${GAMEBOY_DOCTOR_DIR}"

mkdir -p "${ROMS}"
curl -LO https://github.com/c-sp/game-boy-test-roms/releases/download/v7.0/game-boy-test-roms-v7.0.zip
unzip -n game-boy-test-roms-v7.0.zip -d "${ROMS}"
rm game-boy-test-roms-v7.0.zip

git -C "${GAMEBOY_SM83_DIR}" pull || git clone https://github.com/SingleStepTests/sm83 --depth 1 "${GAMEBOY_SM83_DIR}"