#!/usr/bin/env bats
# lib/common.sh のテスト

load '../helpers/stub'

setup() {
  bwqa_test_stub_setup
  source "$BWQA_LIB_DIR/common.sh"
}

teardown() {
  bwqa_test_stub_teardown
}

@test "bwqa_version_ge: 同一バージョンは真" {
  run bwqa_version_ge "0.35.0" "0.35.0"
  [ "$status" -eq 0 ]
}

@test "bwqa_version_ge: パッチバージョンが大きければ真" {
  run bwqa_version_ge "0.35.1" "0.35.0"
  [ "$status" -eq 0 ]
}

@test "bwqa_version_ge: パッチバージョンが小さければ偽" {
  run bwqa_version_ge "0.34.9" "0.35.0"
  [ "$status" -ne 0 ]
}

@test "bwqa_version_ge: マイナーバージョンの桁上がりを正しく比較する(文字列比較だと誤判定する例)" {
  run bwqa_version_ge "0.9.0" "0.35.0"
  [ "$status" -ne 0 ]

  run bwqa_version_ge "0.10.0" "0.9.0"
  [ "$status" -eq 0 ]
}

@test "bwqa_version_ge: メジャーバージョンの違いを比較する" {
  run bwqa_version_ge "1.0.0" "0.35.0"
  [ "$status" -eq 0 ]

  run bwqa_version_ge "0.35.0" "1.0.0"
  [ "$status" -ne 0 ]
}

@test "bwqa_version_ge: 桁数が不足しているバージョン文字列を 0 埋めで扱う" {
  run bwqa_version_ge "0.35" "0.35.0"
  [ "$status" -eq 0 ]

  run bwqa_version_ge "0.35.0" "0.35.1"
  [ "$status" -ne 0 ]
}

# shellcheck disable=SC2034
@test "bwqa_detect_lang: BWQA_LANG=en かつ LANG=ja_JP.UTF-8 のときは BWQA_LANG が最優先され en になる" {
  BWQA_LANG="en"
  LANG="ja_JP.UTF-8"
  run bwqa_detect_lang
  [ "$status" -eq 0 ]
  [ "$output" = "en" ]
}

# shellcheck disable=SC2034
@test "bwqa_detect_lang: BWQA_LANG が unset で LANG=ja_JP.UTF-8 のときは ja になる" {
  unset BWQA_LANG
  LANG="ja_JP.UTF-8"
  run bwqa_detect_lang
  [ "$status" -eq 0 ]
  [ "$output" = "ja" ]
}

# shellcheck disable=SC2034
@test "bwqa_detect_lang: BWQA_LANG と LANG が unset で LC_ALL=ja_JP.UTF-8 のときは ja になる" {
  unset BWQA_LANG
  unset LANG
  LC_ALL="ja_JP.UTF-8"
  run bwqa_detect_lang
  [ "$status" -eq 0 ]
  [ "$output" = "ja" ]
}

# shellcheck disable=SC2034
@test "bwqa_detect_lang: LC_ALL が LANG より優先される (LC_ALL=en_US.UTF-8 かつ LANG=ja_JP.UTF-8 のときは en)" {
  unset BWQA_LANG
  LC_ALL="en_US.UTF-8"
  LANG="ja_JP.UTF-8"
  run bwqa_detect_lang
  [ "$status" -eq 0 ]
  [ "$output" = "en" ]
}

@test "bwqa_detect_lang: BWQA_LANG・LANG・LC_ALL がすべて unset のときは en にフォールバックする" {
  unset BWQA_LANG
  unset LANG
  unset LC_ALL
  run bwqa_detect_lang
  [ "$status" -eq 0 ]
  [ "$output" = "en" ]
}

# shellcheck disable=SC2034
@test "bwqa_detect_lang: BWQA_LANG=fr (未対応言語) のときは en にフォールバックする" {
  BWQA_LANG="fr"
  run bwqa_detect_lang
  [ "$status" -eq 0 ]
  [ "$output" = "en" ]
}

# shellcheck disable=SC2034
@test "bwqa_detect_lang: BWQA_LANG が unset で LANG=en_US.UTF-8 のときは en になる" {
  unset BWQA_LANG
  LANG="en_US.UTF-8"
  run bwqa_detect_lang
  [ "$status" -eq 0 ]
  [ "$output" = "en" ]
}

