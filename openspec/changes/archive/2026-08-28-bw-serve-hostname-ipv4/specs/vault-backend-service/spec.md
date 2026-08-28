## MODIFIED Requirements

### Requirement: 動的ポート割り当てとlocalhost限定バインド
本サービスは、`bw serve` の待受ポートを起動のたびに動的に選択し、`--hostname 127.0.0.1` を指定して外部ネットワークに一切晒してはならない(SHALL NOT)。

#### Scenario: 空きポートが選択される
- **WHEN** `bw serve` を起動する
- **THEN** 使用中でない空きポート番号が選ばれ、`bw serve --hostname 127.0.0.1 --port <選ばれたポート>` として起動される
