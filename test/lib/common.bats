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
