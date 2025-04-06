# Changelog

## [0.18.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.17.0...ccp-v0.18.0) (2025-04-06)


### ⚠ BREAKING CHANGES

* update crates ([#230](https://github.com/fluencelabs/capacity-commitment-prover/issues/230))
* **cu_worker:** send stats over gRPC ([#198](https://github.com/fluencelabs/capacity-commitment-prover/issues/198))
* **ccp:** autodetect CU available logical cores ([#192](https://github.com/fluencelabs/capacity-commitment-prover/issues/192))
* **ccp:** add CU worker binary ([#164](https://github.com/fluencelabs/capacity-commitment-prover/issues/164))
* **rpc:** batching API ([#136](https://github.com/fluencelabs/capacity-commitment-prover/issues/136))
* **rpc:** get rid of unnesessary OrHex
* **shared:** serialize and deserialize hash-like types as strings ([#146](https://github.com/fluencelabs/capacity-commitment-prover/issues/146))
* fix difficulty checking ([#141](https://github.com/fluencelabs/capacity-commitment-prover/issues/141))
* **ccp:** ccp splitting into async parts ([#130](https://github.com/fluencelabs/capacity-commitment-prover/issues/130))
* **ccp:** fine grain dataset generation ([#125](https://github.com/fluencelabs/capacity-commitment-prover/issues/125))
* **config,ccp:** add more config options ([#99](https://github.com/fluencelabs/capacity-commitment-prover/issues/99))
* **config:** allow empty initial utility cores set ([#98](https://github.com/fluencelabs/capacity-commitment-prover/issues/98))
* **rpc:** more ergonomic RPC client ([#102](https://github.com/fluencelabs/capacity-commitment-prover/issues/102))
* **cfg,ccp:** store utility threads pinning to saved state ([#87](https://github.com/fluencelabs/capacity-commitment-prover/issues/87))
* **msr, persistence:** refactor msr ([#79](https://github.com/fluencelabs/capacity-commitment-prover/issues/79))
* **rpc:** add realloc_utility_cores RPC method ([#69](https://github.com/fluencelabs/capacity-commitment-prover/issues/69))
* **ccp,main:** Graceful shutdown ([#77](https://github.com/fluencelabs/capacity-commitment-prover/issues/77))
* **core:** make crossterm optional ([#71](https://github.com/fluencelabs/capacity-commitment-prover/issues/71))
* **cfg:** Rename [http-server] to [rpc-endpoint] ([#56](https://github.com/fluencelabs/capacity-commitment-prover/issues/56))
* **core:** make all runnables pausable ([#25](https://github.com/fluencelabs/capacity-commitment-prover/issues/25))
* **api:** use EpochParameters where possible ([#24](https://github.com/fluencelabs/capacity-commitment-prover/issues/24))
* **rpc:** add limit to `get_proofs_after` method ([#23](https://github.com/fluencelabs/capacity-commitment-prover/issues/23))

### Features

* **api:** use EpochParameters where possible ([#24](https://github.com/fluencelabs/capacity-commitment-prover/issues/24)) ([33af209](https://github.com/fluencelabs/capacity-commitment-prover/commit/33af209d657e52f0bb8cdc920eff39a7c03df225))
* background on_commitment handling ([#51](https://github.com/fluencelabs/capacity-commitment-prover/issues/51)) ([d9deedf](https://github.com/fluencelabs/capacity-commitment-prover/commit/d9deedfc6fb8c4a0a85db94f93a7f349a24043a7))
* **ccp-shared:** add Ord to CUID type ([#90](https://github.com/fluencelabs/capacity-commitment-prover/issues/90)) ([2beb3b0](https://github.com/fluencelabs/capacity-commitment-prover/commit/2beb3b065c58e0e792a742e3378a685b275b53e9))
* **ccp,main:** Graceful shutdown ([#77](https://github.com/fluencelabs/capacity-commitment-prover/issues/77)) ([6b2c6f8](https://github.com/fluencelabs/capacity-commitment-prover/commit/6b2c6f85819ad44d70560a95181b68bdf1323eb5))
* **ccp:** add CU worker binary ([#164](https://github.com/fluencelabs/capacity-commitment-prover/issues/164)) ([7247e29](https://github.com/fluencelabs/capacity-commitment-prover/commit/7247e29105d182aa5a719a7da12e65dac88ab390))
* **ccp:** autodetect CU available logical cores ([#192](https://github.com/fluencelabs/capacity-commitment-prover/issues/192)) ([4a76cb1](https://github.com/fluencelabs/capacity-commitment-prover/commit/4a76cb1a00d96c37c650c5b95abfc65bf707a955))
* **ccp:** ccp splitting into async parts ([#130](https://github.com/fluencelabs/capacity-commitment-prover/issues/130)) ([955c634](https://github.com/fluencelabs/capacity-commitment-prover/commit/955c63428ee8fba6729a349220c6007d07f31ee2))
* **ccp:** fine grain dataset generation ([#125](https://github.com/fluencelabs/capacity-commitment-prover/issues/125)) ([2595e81](https://github.com/fluencelabs/capacity-commitment-prover/commit/2595e81e284b73c50ae5cfc1cbb6e1f6a926a9a7))
* **ccp:** more dev UX improvements ([#177](https://github.com/fluencelabs/capacity-commitment-prover/issues/177)) ([3574a6f](https://github.com/fluencelabs/capacity-commitment-prover/commit/3574a6f065baf0736a68e6bc0df64766457ce4c4))
* **ccp:** persistent state ([#43](https://github.com/fluencelabs/capacity-commitment-prover/issues/43)) ([d864c89](https://github.com/fluencelabs/capacity-commitment-prover/commit/d864c89e438f671a8722b9272dba258befbc4b93))
* **ccp:** Prometheus metrics for I/O ([#150](https://github.com/fluencelabs/capacity-commitment-prover/issues/150)) ([5d611c8](https://github.com/fluencelabs/capacity-commitment-prover/commit/5d611c8af018fbbb32f552b37da341eebc3eabbd))
* **ccp:** reduce CCP memory consumption ([#175](https://github.com/fluencelabs/capacity-commitment-prover/issues/175)) ([36b172b](https://github.com/fluencelabs/capacity-commitment-prover/commit/36b172b2c0a92de728e88791ecc6c5aeb8f4945f))
* **ccp:** reduce disk usage on dataset generation ([#191](https://github.com/fluencelabs/capacity-commitment-prover/issues/191)) ([2ad865b](https://github.com/fluencelabs/capacity-commitment-prover/commit/2ad865b7933a2f55ce6c4bbe411fbd66cabfa38f))
* **ccp:** split off CU threads into a separate crate ([#163](https://github.com/fluencelabs/capacity-commitment-prover/issues/163)) ([6c3c5a8](https://github.com/fluencelabs/capacity-commitment-prover/commit/6c3c5a83dbdc3fb9d8354dcf4472cceda9340785))
* **ccp:** split UtilityThread functionality in two: StatsCollecting and StateOperating [fixes VM-589] ([#129](https://github.com/fluencelabs/capacity-commitment-prover/issues/129)) ([fc1ca55](https://github.com/fluencelabs/capacity-commitment-prover/commit/fc1ca55632bf48933c3244fdd460b6eb032d0631))
* **cfg,ccp:** store utility threads pinning to saved state ([#87](https://github.com/fluencelabs/capacity-commitment-prover/issues/87)) ([1b2ccd5](https://github.com/fluencelabs/capacity-commitment-prover/commit/1b2ccd5982f59783e06f12f5e037b40f3d0c9c0c))
* **cfg:** default loglevel is info ([#193](https://github.com/fluencelabs/capacity-commitment-prover/issues/193)) ([5e37254](https://github.com/fluencelabs/capacity-commitment-prover/commit/5e37254f959601018ba1cdbe2436b4a707f4576a))
* **cfg:** make `[optimizations]` optional ([#61](https://github.com/fluencelabs/capacity-commitment-prover/issues/61)) ([6604f71](https://github.com/fluencelabs/capacity-commitment-prover/commit/6604f71ebc057653d723e8100f68495e87365145))
* **cli:** CLI checks that directories exist ([#14](https://github.com/fluencelabs/capacity-commitment-prover/issues/14)) ([5c5a69c](https://github.com/fluencelabs/capacity-commitment-prover/commit/5c5a69c59e18ebbd0f3c5146a784e640c0ae457f))
* collect hashrate ([#44](https://github.com/fluencelabs/capacity-commitment-prover/issues/44)) ([74da415](https://github.com/fluencelabs/capacity-commitment-prover/commit/74da41560e96d34c24c127a867506859e42db5c6))
* **config,ccp:** add more config options ([#99](https://github.com/fluencelabs/capacity-commitment-prover/issues/99)) ([0188711](https://github.com/fluencelabs/capacity-commitment-prover/commit/0188711cc7476a4cccf47af68cc64745c3bca9a1))
* **config:** add logs.cumulative_hashrate_update_interval_secs ([#151](https://github.com/fluencelabs/capacity-commitment-prover/issues/151)) ([7f2926a](https://github.com/fluencelabs/capacity-commitment-prover/commit/7f2926afedac4155dafebd94f0a73a57643d2b7b))
* **config:** allow empty initial utility cores set ([#98](https://github.com/fluencelabs/capacity-commitment-prover/issues/98)) ([e5b73f8](https://github.com/fluencelabs/capacity-commitment-prover/commit/e5b73f8fdc44f5c63094d2e118f29a2c0f8f3570))
* **core:** add result hash into CCProof ([#22](https://github.com/fluencelabs/capacity-commitment-prover/issues/22)) ([1e61762](https://github.com/fluencelabs/capacity-commitment-prover/commit/1e6176228017e626cdd09ddbc7659cd64d3cdf29))
* **core:** make all runnables pausable ([#25](https://github.com/fluencelabs/capacity-commitment-prover/issues/25)) ([04aa11b](https://github.com/fluencelabs/capacity-commitment-prover/commit/04aa11bf0b78dd82086fcac10e8ff011d8abd055))
* **core:** make crossterm optional ([#71](https://github.com/fluencelabs/capacity-commitment-prover/issues/71)) ([b8d18fc](https://github.com/fluencelabs/capacity-commitment-prover/commit/b8d18fcc7dc82e72c9e7cfacc1ac41e14e95f4a5))
* **core:** propate errors, introduce utility thread ([#31](https://github.com/fluencelabs/capacity-commitment-prover/issues/31)) ([d45e762](https://github.com/fluencelabs/capacity-commitment-prover/commit/d45e7626f62e8034964265d301c59b135a3c0a13))
* **cpu-utils:** expose physical_cores ([17479d1](https://github.com/fluencelabs/capacity-commitment-prover/commit/17479d142e6c7344ff3f0e60cd9e6628c3fd5961))
* **cpu-utils:** introduced new abstraction for pinning and topology fetching ([#134](https://github.com/fluencelabs/capacity-commitment-prover/issues/134)) ([696720d](https://github.com/fluencelabs/capacity-commitment-prover/commit/696720da12f559fe2e43ebb7339c0c5878ad89e0))
* **cu_worker:** cpu worker stats to console ([#196](https://github.com/fluencelabs/capacity-commitment-prover/issues/196)) ([ffba666](https://github.com/fluencelabs/capacity-commitment-prover/commit/ffba666643f8f9a5e8c5abda2cda82ce0f32d79e))
* **cu_worker:** send stats over gRPC ([#198](https://github.com/fluencelabs/capacity-commitment-prover/issues/198)) ([23c7cda](https://github.com/fluencelabs/capacity-commitment-prover/commit/23c7cdab74ef3f48e84935c2fff964c9275e3819))
* **cu:** O_DIRECT for dump file ([#211](https://github.com/fluencelabs/capacity-commitment-prover/issues/211)) ([0cd0722](https://github.com/fluencelabs/capacity-commitment-prover/commit/0cd0722a2f75f850e940bcce758f43e812a47bd0))
* implement thread pinning ([#11](https://github.com/fluencelabs/capacity-commitment-prover/issues/11)) ([9dd9eff](https://github.com/fluencelabs/capacity-commitment-prover/commit/9dd9effbb3a61ca08da78ca65175077b9b67728d))
* introduce configs ([#54](https://github.com/fluencelabs/capacity-commitment-prover/issues/54)) ([de3eea2](https://github.com/fluencelabs/capacity-commitment-prover/commit/de3eea274191b5ae6f6a1ad0415d8d758f25f992))
* **msr, persistence:** refactor msr ([#79](https://github.com/fluencelabs/capacity-commitment-prover/issues/79)) ([8510ac7](https://github.com/fluencelabs/capacity-commitment-prover/commit/8510ac78bb63f5959832c8e1bcb8047f8a4be2c6))
* **optimization:** CPU cache control to enable hashrate optimizations [fixes VM-488] ([5de0546](https://github.com/fluencelabs/capacity-commitment-prover/commit/5de05462da4444f7fd3633fecaecd82c854a1618))
* optional prometheus endpoint ([#55](https://github.com/fluencelabs/capacity-commitment-prover/issues/55)) ([539cc37](https://github.com/fluencelabs/capacity-commitment-prover/commit/539cc3794ea175d96aa5e64599ed113b202c1577))
* refactor ProvingThread ([#10](https://github.com/fluencelabs/capacity-commitment-prover/issues/10)) ([7751e6e](https://github.com/fluencelabs/capacity-commitment-prover/commit/7751e6e488d158cc8ac09e57bcedc1b7e7f7ae5c))
* **rpc-client:** add gRPC descriptor set ([#189](https://github.com/fluencelabs/capacity-commitment-prover/issues/189)) ([92d0fbb](https://github.com/fluencelabs/capacity-commitment-prover/commit/92d0fbbf3174ad3300766515d918d763944cb7d7))
* **rpc:** add limit to `get_proofs_after` method ([#23](https://github.com/fluencelabs/capacity-commitment-prover/issues/23)) ([86b5579](https://github.com/fluencelabs/capacity-commitment-prover/commit/86b55795bfb1bdea85cd312606eaadd0b2a4cfcd))
* **rpc:** add realloc_utility_cores RPC method ([#69](https://github.com/fluencelabs/capacity-commitment-prover/issues/69)) ([a35fc11](https://github.com/fluencelabs/capacity-commitment-prover/commit/a35fc11e205cda2d6cd36b871b760dcac7ddf666))
* **rpc:** batching API ([#136](https://github.com/fluencelabs/capacity-commitment-prover/issues/136)) ([08519a7](https://github.com/fluencelabs/capacity-commitment-prover/commit/08519a7b997adb01e133902b4432f8418bcb35d1))
* **rpc:** more ergonomic RPC client ([#102](https://github.com/fluencelabs/capacity-commitment-prover/issues/102)) ([0146973](https://github.com/fluencelabs/capacity-commitment-prover/commit/01469732b22996e49c18a9d6fa63acf9aeee2c1c))
* **rpc:** new CCPRpcClient constructors ([#120](https://github.com/fluencelabs/capacity-commitment-prover/issues/120)) ([ba685ae](https://github.com/fluencelabs/capacity-commitment-prover/commit/ba685ae1b2a189aa945d37576f02dc7d6eab41cf))
* **rpc:** OrHex type uses FromHex and unhex method ([#15](https://github.com/fluencelabs/capacity-commitment-prover/issues/15)) ([c477aec](https://github.com/fluencelabs/capacity-commitment-prover/commit/c477aecd78cc580593bbfc8e30e3b41e6959b44d))
* **shared:** Add `const` to many `ccp-shared` methods ([#204](https://github.com/fluencelabs/capacity-commitment-prover/issues/204)) ([b4bcb93](https://github.com/fluencelabs/capacity-commitment-prover/commit/b4bcb93d8fb43cbb6618d03c94d2ae4e6e7eced0))
* **shared:** add an optional `arbitrary` support ([#183](https://github.com/fluencelabs/capacity-commitment-prover/issues/183)) ([3e17b19](https://github.com/fluencelabs/capacity-commitment-prover/commit/3e17b19ad7760745cc2a5c85b510bcc0d7674fb4))
* **shared:** implement Display for certain types ([#63](https://github.com/fluencelabs/capacity-commitment-prover/issues/63)) ([dccf7e5](https://github.com/fluencelabs/capacity-commitment-prover/commit/dccf7e5cdba05ae77f752592e2be9da65277aa5c))
* **shared:** implement FromStr for shared types ([#121](https://github.com/fluencelabs/capacity-commitment-prover/issues/121)) ([001f8bc](https://github.com/fluencelabs/capacity-commitment-prover/commit/001f8bceb1cf25df4a06882ee7476e7d1cfcfdbf))
* **shared:** implement ToHex for various types ([#28](https://github.com/fluencelabs/capacity-commitment-prover/issues/28)) ([f82d08b](https://github.com/fluencelabs/capacity-commitment-prover/commit/f82d08b0a1dbc77a25b5bd7f2f91976f36561ad6))
* **shared:** opaque types ([#13](https://github.com/fluencelabs/capacity-commitment-prover/issues/13)) ([f3f8e8e](https://github.com/fluencelabs/capacity-commitment-prover/commit/f3f8e8e3720bf8e2c10c91cbd732d873f2a0878d))
* **shared:** readable Debug for binary types ([#115](https://github.com/fluencelabs/capacity-commitment-prover/issues/115)) ([2553a77](https://github.com/fluencelabs/capacity-commitment-prover/commit/2553a772f99ae6c4423e1a22b0c61d19d739d22d))
* **shared:** serialize and deserialize hash-like types as strings ([#146](https://github.com/fluencelabs/capacity-commitment-prover/issues/146)) ([8b4135c](https://github.com/fluencelabs/capacity-commitment-prover/commit/8b4135c4887013ae5b9bcb8ae0c75d169e46f873))
* single state dir ([#50](https://github.com/fluencelabs/capacity-commitment-prover/issues/50)) ([87f0d36](https://github.com/fluencelabs/capacity-commitment-prover/commit/87f0d365626552a54f1d2dac8d04b36e352cbc1b))
* Tune different parameters ([#209](https://github.com/fluencelabs/capacity-commitment-prover/issues/209)) ([2c0665b](https://github.com/fluencelabs/capacity-commitment-prover/commit/2c0665b446045e226d8de5a4df196ee14cf952e9))
* use 2 GiB randomx ([#227](https://github.com/fluencelabs/capacity-commitment-prover/issues/227)) ([3f3f92a](https://github.com/fluencelabs/capacity-commitment-prover/commit/3f3f92a36323ebf108dab9a45a45177e51355e16))
* **utility-thread:** SyncProvingThread now explicitly notifies UtilityThread with the current epoch building the cache up [fixes VM-594] ([#128](https://github.com/fluencelabs/capacity-commitment-prover/issues/128)) ([fb61ccd](https://github.com/fluencelabs/capacity-commitment-prover/commit/fb61ccd920feed9211bca8d0f618275100c10ccd))


### Bug Fixes

* **cargo:** move ccp-test-utils dependency to dev-dependencies of ccp ([#108](https://github.com/fluencelabs/capacity-commitment-prover/issues/108)) ([40d4205](https://github.com/fluencelabs/capacity-commitment-prover/commit/40d420526dcb02c0a9f94dffc2bb99264fcd71df))
* **ccp:** keep proofs on restart ([#148](https://github.com/fluencelabs/capacity-commitment-prover/issues/148)) ([2c2e767](https://github.com/fluencelabs/capacity-commitment-prover/commit/2c2e767208cbc55c3a8dd4cf55a0b68f56c71b94))
* **ccp:** log start and dataset generation reason ([#167](https://github.com/fluencelabs/capacity-commitment-prover/issues/167)) ([6f46d2e](https://github.com/fluencelabs/capacity-commitment-prover/commit/6f46d2e2f7049868b22113011883dda685ff01de))
* **ccp:** manage state in the utility thread ([#127](https://github.com/fluencelabs/capacity-commitment-prover/issues/127)) ([dc5e3c4](https://github.com/fluencelabs/capacity-commitment-prover/commit/dc5e3c47b5ea3810c82d48b33eb462fc7f0ce543))
* **ccp:** more extensive proof lifecycle logging ([#169](https://github.com/fluencelabs/capacity-commitment-prover/issues/169)) ([5df7e03](https://github.com/fluencelabs/capacity-commitment-prover/commit/5df7e03bb421d7070f20959c78428aa0a82e739f))
* **ccp:** use random temporary file name ([#78](https://github.com/fluencelabs/capacity-commitment-prover/issues/78)) ([f8dd6b4](https://github.com/fluencelabs/capacity-commitment-prover/commit/f8dd6b458033476ced2fb00b3ca59c6310e88998))
* **cfg:** Rename [http-server] to [rpc-endpoint] ([#56](https://github.com/fluencelabs/capacity-commitment-prover/issues/56)) ([ca81b4f](https://github.com/fluencelabs/capacity-commitment-prover/commit/ca81b4f16958bff759d9b0f78dbc2724d6cfb419))
* **client:** new doens't need self ([#12](https://github.com/fluencelabs/capacity-commitment-prover/issues/12)) ([49bf269](https://github.com/fluencelabs/capacity-commitment-prover/commit/49bf2698d46cf720587d3430329a6ab1401e3c66))
* **cli:** fix dir detection ([#17](https://github.com/fluencelabs/capacity-commitment-prover/issues/17)) ([3ea8cee](https://github.com/fluencelabs/capacity-commitment-prover/commit/3ea8cee84668dbf141af2e2c4ef269294c44656b))
* **config:** Correct Default for UnresolvedOptimizations ([#62](https://github.com/fluencelabs/capacity-commitment-prover/issues/62)) ([3470f68](https://github.com/fluencelabs/capacity-commitment-prover/commit/3470f68135c72bf22405dd19f46218248d800db8))
* **config:** default config for logs and state ([#92](https://github.com/fluencelabs/capacity-commitment-prover/issues/92)) ([7f062fc](https://github.com/fluencelabs/capacity-commitment-prover/commit/7f062fc528dea52b0a502d6d7b6fbfb89e773620))
* **config:** fix default config field names, add state dir defaults, create state dir ([#89](https://github.com/fluencelabs/capacity-commitment-prover/issues/89)) ([2684980](https://github.com/fluencelabs/capacity-commitment-prover/commit/2684980ec691c5ee278f1317075bb22d2bc5f057))
* **config:** fix tokio config keys case ([#149](https://github.com/fluencelabs/capacity-commitment-prover/issues/149)) ([e8a10e5](https://github.com/fluencelabs/capacity-commitment-prover/commit/e8a10e5eeeb87b58a45d19c2e95a7e257aec1e3f))
* **cpu-utils:** re-export `nonempty::NonEmpty` ([#154](https://github.com/fluencelabs/capacity-commitment-prover/issues/154)) ([1e8a860](https://github.com/fluencelabs/capacity-commitment-prover/commit/1e8a8600604bcc9ab2f43139be0c72371c5327ff))
* **crates:** Fix typo ([#8](https://github.com/fluencelabs/capacity-commitment-prover/issues/8)) ([f12341c](https://github.com/fluencelabs/capacity-commitment-prover/commit/f12341c01d0a08e3a2de3bf34cec507200cbf19c))
* **cu_worker:** update oom_score_adj file ([#207](https://github.com/fluencelabs/capacity-commitment-prover/issues/207)) ([2a9e89b](https://github.com/fluencelabs/capacity-commitment-prover/commit/2a9e89bd0b02a4d0c3ddac6e7f72a17363d1d469))
* **cu:** display numeric errno code ([#220](https://github.com/fluencelabs/capacity-commitment-prover/issues/220)) ([038bf7d](https://github.com/fluencelabs/capacity-commitment-prover/commit/038bf7d523d838c0974d871178cbf1b8d99448a1))
* **cu:** dump memory to disk in chunks with sync ([#213](https://github.com/fluencelabs/capacity-commitment-prover/issues/213)) ([58f2b44](https://github.com/fluencelabs/capacity-commitment-prover/commit/58f2b44391efb27009fd73a02cfb9f80bade59d3))
* **cu:** mmap over mmap ([#224](https://github.com/fluencelabs/capacity-commitment-prover/issues/224)) ([0c7643b](https://github.com/fluencelabs/capacity-commitment-prover/commit/0c7643bbe9b8ccbecfaf710beb555577c8129573))
* CUProver wasn't setting correct pinned core after repinning ([#114](https://github.com/fluencelabs/capacity-commitment-prover/issues/114)) ([ee56885](https://github.com/fluencelabs/capacity-commitment-prover/commit/ee568852d2df3c705d5583aaa454ce264a59b3cc))
* **cu:** try MADV_PAGEOUT on Linux ([#222](https://github.com/fluencelabs/capacity-commitment-prover/issues/222)) ([460fa27](https://github.com/fluencelabs/capacity-commitment-prover/commit/460fa2731d8fc86e381fa9c5cf1064f5fd2b11db))
* **cu:** Use MADV_REMOVE on Linux ([#217](https://github.com/fluencelabs/capacity-commitment-prover/issues/217)) ([35c1fd8](https://github.com/fluencelabs/capacity-commitment-prover/commit/35c1fd8a45dd5e058c0fff3f6c58a246e45d8714))
* decouple a public crate from a private one ([#200](https://github.com/fluencelabs/capacity-commitment-prover/issues/200)) ([205e772](https://github.com/fluencelabs/capacity-commitment-prover/commit/205e772dc4f251fb49d01fafe86329f3b629c3c9))
* decouple a public crate from private one ([205e772](https://github.com/fluencelabs/capacity-commitment-prover/commit/205e772dc4f251fb49d01fafe86329f3b629c3c9))
* fix channel closing ([#202](https://github.com/fluencelabs/capacity-commitment-prover/issues/202)) ([4cc9d3d](https://github.com/fluencelabs/capacity-commitment-prover/commit/4cc9d3dc06a1b4cd58279fe9a72e4b390b98a48f))
* fix cross compilation to arm ([#57](https://github.com/fluencelabs/capacity-commitment-prover/issues/57)) ([f1c85a0](https://github.com/fluencelabs/capacity-commitment-prover/commit/f1c85a015ec1a2392b1a20a1f0279da79e96313c))
* fix difficulty checking ([#141](https://github.com/fluencelabs/capacity-commitment-prover/issues/141)) ([95f8cf8](https://github.com/fluencelabs/capacity-commitment-prover/commit/95f8cf8c7594262bf9a4b699b2920115178e4089))
* **hashrate:** fix instant hashrate computation ([#111](https://github.com/fluencelabs/capacity-commitment-prover/issues/111)) ([36efb58](https://github.com/fluencelabs/capacity-commitment-prover/commit/36efb58f255640605516263bd748f3d33a303386))
* **main:** do not try to pin to an empty CPU set ([#126](https://github.com/fluencelabs/capacity-commitment-prover/issues/126)) ([4106bd7](https://github.com/fluencelabs/capacity-commitment-prover/commit/4106bd7b962079aebcc657a41f4ac80e25e9c352))
* **msr:** the code now compiles on MacOs ([#41](https://github.com/fluencelabs/capacity-commitment-prover/issues/41)) ([a7241a9](https://github.com/fluencelabs/capacity-commitment-prover/commit/a7241a9bf169ed6687df32a6cf7710df4fe4ba07))
* refine CLI interface and fix README ([#59](https://github.com/fluencelabs/capacity-commitment-prover/issues/59)) ([93027df](https://github.com/fluencelabs/capacity-commitment-prover/commit/93027df80c168058bad0ec0a3ffaf005abb63dc8))
* refine config saving ([#60](https://github.com/fluencelabs/capacity-commitment-prover/issues/60)) ([a21321d](https://github.com/fluencelabs/capacity-commitment-prover/commit/a21321d1251c22dd1cfd51962ccf8fc2d6a181f2))
* remove excess linux attrs ([#58](https://github.com/fluencelabs/capacity-commitment-prover/issues/58)) ([06c7359](https://github.com/fluencelabs/capacity-commitment-prover/commit/06c73595ad2ac740fe4bd8ae65b5afbc941ab2c5))
* **rpc:** add missing get_proofs_after method to client ([#21](https://github.com/fluencelabs/capacity-commitment-prover/issues/21)) ([92de08e](https://github.com/fluencelabs/capacity-commitment-prover/commit/92de08e8c457204bf3ff80354bbedbf3b8e5d7e7))
* **rpc:** get rid of unnesessary OrHex ([8b4135c](https://github.com/fluencelabs/capacity-commitment-prover/commit/8b4135c4887013ae5b9bcb8ae0c75d169e46f873))
* **rpc:** get_proofs_after returns empty list on busy lock ([#81](https://github.com/fluencelabs/capacity-commitment-prover/issues/81)) ([84b8e95](https://github.com/fluencelabs/capacity-commitment-prover/commit/84b8e956dca1e946d35598605c2c27330a2ceb54))
* **rpc:** make pub use ClientError ([#131](https://github.com/fluencelabs/capacity-commitment-prover/issues/131)) ([270a51d](https://github.com/fluencelabs/capacity-commitment-prover/commit/270a51de9ba388ae9fa43404d1aaa0790e241836))
* **rpc:** reexport jsonrpsee::..::ErrorObjectOwned ([#145](https://github.com/fluencelabs/capacity-commitment-prover/issues/145)) ([5a0ed57](https://github.com/fluencelabs/capacity-commitment-prover/commit/5a0ed5757c45cad4f2beea49e6883b3d45cdce73))
* **rpc:** Use ProofIdx type in API types ([#20](https://github.com/fluencelabs/capacity-commitment-prover/issues/20)) ([2c365ec](https://github.com/fluencelabs/capacity-commitment-prover/commit/2c365ec658bf58dad861f23af4fb9bdc4188c6ac))
* **shared:** fix crate publishing ([#181](https://github.com/fluencelabs/capacity-commitment-prover/issues/181)) ([bad46e7](https://github.com/fluencelabs/capacity-commitment-prover/commit/bad46e7c0b980e8920614276602ebf0551689aff))
* **shared:** implement Display for Difficulty ([#66](https://github.com/fluencelabs/capacity-commitment-prover/issues/66)) ([0be20a5](https://github.com/fluencelabs/capacity-commitment-prover/commit/0be20a5c461f9191d8e83fdef33d376203565562))
* stop prover threads simultaneously ([#93](https://github.com/fluencelabs/capacity-commitment-prover/issues/93)) ([dc183fd](https://github.com/fluencelabs/capacity-commitment-prover/commit/dc183fd4eeb12a3870bed27f319fa5882fc57854))
* stop provers while cleaning proof cache up ([#27](https://github.com/fluencelabs/capacity-commitment-prover/issues/27)) ([0dee01c](https://github.com/fluencelabs/capacity-commitment-prover/commit/0dee01c6ad3d06d866b568688e6591afe176f7a6))
* **test-utils:** enable publishing ([#138](https://github.com/fluencelabs/capacity-commitment-prover/issues/138)) ([9828e57](https://github.com/fluencelabs/capacity-commitment-prover/commit/9828e57f0cd62cdd9d39a67908f024d24c3ca2de))
* update crates ([#230](https://github.com/fluencelabs/capacity-commitment-prover/issues/230)) ([f4476a9](https://github.com/fluencelabs/capacity-commitment-prover/commit/f4476a96eea830f895e7cc96a105725f1550d6d5))

## [0.17.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.16.12...ccp-v0.17.0) (2025-04-06)


### ⚠ BREAKING CHANGES

* update crates ([#230](https://github.com/fluencelabs/capacity-commitment-prover/issues/230))

### Bug Fixes

* update crates ([#230](https://github.com/fluencelabs/capacity-commitment-prover/issues/230)) ([f4476a9](https://github.com/fluencelabs/capacity-commitment-prover/commit/f4476a96eea830f895e7cc96a105725f1550d6d5))

## [0.16.12](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.16.11...ccp-v0.16.12) (2025-03-14)


### Features

* use 2 GiB randomx ([#227](https://github.com/fluencelabs/capacity-commitment-prover/issues/227)) ([3f3f92a](https://github.com/fluencelabs/capacity-commitment-prover/commit/3f3f92a36323ebf108dab9a45a45177e51355e16))

## [0.16.11](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.16.10...ccp-v0.16.11) (2025-03-12)


### Bug Fixes

* **cu:** mmap over mmap ([#224](https://github.com/fluencelabs/capacity-commitment-prover/issues/224)) ([0c7643b](https://github.com/fluencelabs/capacity-commitment-prover/commit/0c7643bbe9b8ccbecfaf710beb555577c8129573))

## [0.16.10](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.16.9...ccp-v0.16.10) (2025-03-11)


### Bug Fixes

* **cu:** try MADV_PAGEOUT on Linux ([#222](https://github.com/fluencelabs/capacity-commitment-prover/issues/222)) ([460fa27](https://github.com/fluencelabs/capacity-commitment-prover/commit/460fa2731d8fc86e381fa9c5cf1064f5fd2b11db))

## [0.16.9](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.16.8...ccp-v0.16.9) (2025-03-11)


### Bug Fixes

* **cu:** display numeric errno code ([#220](https://github.com/fluencelabs/capacity-commitment-prover/issues/220)) ([038bf7d](https://github.com/fluencelabs/capacity-commitment-prover/commit/038bf7d523d838c0974d871178cbf1b8d99448a1))

## [0.16.8](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.16.7...ccp-v0.16.8) (2025-03-11)


### Bug Fixes

* **cu:** Use MADV_REMOVE on Linux ([#217](https://github.com/fluencelabs/capacity-commitment-prover/issues/217)) ([35c1fd8](https://github.com/fluencelabs/capacity-commitment-prover/commit/35c1fd8a45dd5e058c0fff3f6c58a246e45d8714))

## [0.16.7](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.16.6...ccp-v0.16.7) (2025-03-10)


### Bug Fixes

* **cu:** dump memory to disk in chunks with sync ([#213](https://github.com/fluencelabs/capacity-commitment-prover/issues/213)) ([58f2b44](https://github.com/fluencelabs/capacity-commitment-prover/commit/58f2b44391efb27009fd73a02cfb9f80bade59d3))

## [0.16.6](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.16.5...ccp-v0.16.6) (2025-03-10)


### Features

* **cu:** O_DIRECT for dump file ([#211](https://github.com/fluencelabs/capacity-commitment-prover/issues/211)) ([0cd0722](https://github.com/fluencelabs/capacity-commitment-prover/commit/0cd0722a2f75f850e940bcce758f43e812a47bd0))

## [0.16.5](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.16.4...ccp-v0.16.5) (2025-03-09)


### Features

* Tune different parameters ([#209](https://github.com/fluencelabs/capacity-commitment-prover/issues/209)) ([2c0665b](https://github.com/fluencelabs/capacity-commitment-prover/commit/2c0665b446045e226d8de5a4df196ee14cf952e9))

## [0.16.4](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.16.3...ccp-v0.16.4) (2025-03-09)


### Bug Fixes

* **cu_worker:** update oom_score_adj file ([#207](https://github.com/fluencelabs/capacity-commitment-prover/issues/207)) ([2a9e89b](https://github.com/fluencelabs/capacity-commitment-prover/commit/2a9e89bd0b02a4d0c3ddac6e7f72a17363d1d469))

## [0.16.3](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.16.2...ccp-v0.16.3) (2025-03-03)


### Features

* **shared:** Add `const` to many `ccp-shared` methods ([#204](https://github.com/fluencelabs/capacity-commitment-prover/issues/204)) ([b4bcb93](https://github.com/fluencelabs/capacity-commitment-prover/commit/b4bcb93d8fb43cbb6618d03c94d2ae4e6e7eced0))

## [0.16.2](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.16.1...ccp-v0.16.2) (2025-02-06)


### Bug Fixes

* fix channel closing ([#202](https://github.com/fluencelabs/capacity-commitment-prover/issues/202)) ([4cc9d3d](https://github.com/fluencelabs/capacity-commitment-prover/commit/4cc9d3dc06a1b4cd58279fe9a72e4b390b98a48f))

## [0.16.1](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.16.0...ccp-v0.16.1) (2025-01-09)


### Bug Fixes

* decouple a public crate from a private one ([#200](https://github.com/fluencelabs/capacity-commitment-prover/issues/200)) ([205e772](https://github.com/fluencelabs/capacity-commitment-prover/commit/205e772dc4f251fb49d01fafe86329f3b629c3c9))

## [0.16.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.15.0...ccp-v0.16.0) (2025-01-09)


### ⚠ BREAKING CHANGES

* **cu_worker:** send stats over gRPC ([#198](https://github.com/fluencelabs/capacity-commitment-prover/issues/198))

### Features

* **cu_worker:** cpu worker stats to console ([#196](https://github.com/fluencelabs/capacity-commitment-prover/issues/196)) ([ffba666](https://github.com/fluencelabs/capacity-commitment-prover/commit/ffba666643f8f9a5e8c5abda2cda82ce0f32d79e))
* **cu_worker:** send stats over gRPC ([#198](https://github.com/fluencelabs/capacity-commitment-prover/issues/198)) ([23c7cda](https://github.com/fluencelabs/capacity-commitment-prover/commit/23c7cdab74ef3f48e84935c2fff964c9275e3819))

## [0.15.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.14.3...ccp-v0.15.0) (2024-12-19)


### ⚠ BREAKING CHANGES

* **ccp:** autodetect CU available logical cores ([#192](https://github.com/fluencelabs/capacity-commitment-prover/issues/192))

### Features

* **ccp:** autodetect CU available logical cores ([#192](https://github.com/fluencelabs/capacity-commitment-prover/issues/192)) ([4a76cb1](https://github.com/fluencelabs/capacity-commitment-prover/commit/4a76cb1a00d96c37c650c5b95abfc65bf707a955))
* **ccp:** reduce disk usage on dataset generation ([#191](https://github.com/fluencelabs/capacity-commitment-prover/issues/191)) ([2ad865b](https://github.com/fluencelabs/capacity-commitment-prover/commit/2ad865b7933a2f55ce6c4bbe411fbd66cabfa38f))
* **cfg:** default loglevel is info ([#193](https://github.com/fluencelabs/capacity-commitment-prover/issues/193)) ([5e37254](https://github.com/fluencelabs/capacity-commitment-prover/commit/5e37254f959601018ba1cdbe2436b4a707f4576a))

## [0.14.3](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.14.2...ccp-v0.14.3) (2024-12-10)


### Features

* **rpc-client:** add gRPC descriptor set ([#189](https://github.com/fluencelabs/capacity-commitment-prover/issues/189)) ([92d0fbb](https://github.com/fluencelabs/capacity-commitment-prover/commit/92d0fbbf3174ad3300766515d918d763944cb7d7))

## [0.14.2](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.14.1...ccp-v0.14.2) (2024-11-15)


### Features

* **shared:** add an optional `arbitrary` support ([#183](https://github.com/fluencelabs/capacity-commitment-prover/issues/183)) ([3e17b19](https://github.com/fluencelabs/capacity-commitment-prover/commit/3e17b19ad7760745cc2a5c85b510bcc0d7674fb4))

## [0.14.1](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.14.0...ccp-v0.14.1) (2024-10-08)


### Bug Fixes

* **shared:** fix crate publishing ([#181](https://github.com/fluencelabs/capacity-commitment-prover/issues/181)) ([bad46e7](https://github.com/fluencelabs/capacity-commitment-prover/commit/bad46e7c0b980e8920614276602ebf0551689aff))

## [0.14.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.13.4...ccp-v0.14.0) (2024-10-08)


### ⚠ BREAKING CHANGES

* **ccp:** add CU worker binary ([#164](https://github.com/fluencelabs/capacity-commitment-prover/issues/164))

### Features

* **ccp:** add CU worker binary ([#164](https://github.com/fluencelabs/capacity-commitment-prover/issues/164)) ([7247e29](https://github.com/fluencelabs/capacity-commitment-prover/commit/7247e29105d182aa5a719a7da12e65dac88ab390))

## [0.13.4](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.13.3...ccp-v0.13.4) (2024-10-02)


### Features

* **ccp:** more dev UX improvements ([#177](https://github.com/fluencelabs/capacity-commitment-prover/issues/177)) ([3574a6f](https://github.com/fluencelabs/capacity-commitment-prover/commit/3574a6f065baf0736a68e6bc0df64766457ce4c4))

## [0.13.3](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.13.2...ccp-v0.13.3) (2024-09-19)


### Features

* **ccp:** reduce CCP memory consumption ([#175](https://github.com/fluencelabs/capacity-commitment-prover/issues/175)) ([36b172b](https://github.com/fluencelabs/capacity-commitment-prover/commit/36b172b2c0a92de728e88791ecc6c5aeb8f4945f))

## [0.13.2](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.13.1...ccp-v0.13.2) (2024-09-12)


### Bug Fixes

* **ccp:** more extensive proof lifecycle logging ([#169](https://github.com/fluencelabs/capacity-commitment-prover/issues/169)) ([5df7e03](https://github.com/fluencelabs/capacity-commitment-prover/commit/5df7e03bb421d7070f20959c78428aa0a82e739f))

## [0.13.1](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.13.0...ccp-v0.13.1) (2024-09-11)


### Features

* **ccp:** split off CU threads into a separate crate ([#163](https://github.com/fluencelabs/capacity-commitment-prover/issues/163)) ([6c3c5a8](https://github.com/fluencelabs/capacity-commitment-prover/commit/6c3c5a83dbdc3fb9d8354dcf4472cceda9340785))


### Bug Fixes

* **ccp:** log start and dataset generation reason ([#167](https://github.com/fluencelabs/capacity-commitment-prover/issues/167)) ([6f46d2e](https://github.com/fluencelabs/capacity-commitment-prover/commit/6f46d2e2f7049868b22113011883dda685ff01de))

## [0.13.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.12.1...ccp-v0.13.0) (2024-07-05)


### ⚠ BREAKING CHANGES

* **rpc:** batching API ([#136](https://github.com/fluencelabs/capacity-commitment-prover/issues/136))

### Features

* **rpc:** batching API ([#136](https://github.com/fluencelabs/capacity-commitment-prover/issues/136)) ([08519a7](https://github.com/fluencelabs/capacity-commitment-prover/commit/08519a7b997adb01e133902b4432f8418bcb35d1))

## [0.12.1](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.12.0...ccp-v0.12.1) (2024-07-02)


### Features

* **ccp:** Prometheus metrics for I/O ([#150](https://github.com/fluencelabs/capacity-commitment-prover/issues/150)) ([5d611c8](https://github.com/fluencelabs/capacity-commitment-prover/commit/5d611c8af018fbbb32f552b37da341eebc3eabbd))

## [0.12.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.11.0...ccp-v0.12.0) (2024-07-02)


### ⚠ BREAKING CHANGES

* **rpc:** get rid of unnesessary OrHex
* **shared:** serialize and deserialize hash-like types as strings ([#146](https://github.com/fluencelabs/capacity-commitment-prover/issues/146))

### Features

* **config:** add logs.cumulative_hashrate_update_interval_secs ([#151](https://github.com/fluencelabs/capacity-commitment-prover/issues/151)) ([7f2926a](https://github.com/fluencelabs/capacity-commitment-prover/commit/7f2926afedac4155dafebd94f0a73a57643d2b7b))
* **shared:** serialize and deserialize hash-like types as strings ([#146](https://github.com/fluencelabs/capacity-commitment-prover/issues/146)) ([8b4135c](https://github.com/fluencelabs/capacity-commitment-prover/commit/8b4135c4887013ae5b9bcb8ae0c75d169e46f873))


### Bug Fixes

* **ccp:** keep proofs on restart ([#148](https://github.com/fluencelabs/capacity-commitment-prover/issues/148)) ([2c2e767](https://github.com/fluencelabs/capacity-commitment-prover/commit/2c2e767208cbc55c3a8dd4cf55a0b68f56c71b94))
* **config:** fix tokio config keys case ([#149](https://github.com/fluencelabs/capacity-commitment-prover/issues/149)) ([e8a10e5](https://github.com/fluencelabs/capacity-commitment-prover/commit/e8a10e5eeeb87b58a45d19c2e95a7e257aec1e3f))
* **cpu-utils:** re-export `nonempty::NonEmpty` ([#154](https://github.com/fluencelabs/capacity-commitment-prover/issues/154)) ([1e8a860](https://github.com/fluencelabs/capacity-commitment-prover/commit/1e8a8600604bcc9ab2f43139be0c72371c5327ff))
* **rpc:** get rid of unnesessary OrHex ([8b4135c](https://github.com/fluencelabs/capacity-commitment-prover/commit/8b4135c4887013ae5b9bcb8ae0c75d169e46f873))

## [0.11.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.10.2...ccp-v0.11.0) (2024-06-12)


### ⚠ BREAKING CHANGES

* fix difficulty checking ([#141](https://github.com/fluencelabs/capacity-commitment-prover/issues/141))

### Bug Fixes

* fix difficulty checking ([#141](https://github.com/fluencelabs/capacity-commitment-prover/issues/141)) ([95f8cf8](https://github.com/fluencelabs/capacity-commitment-prover/commit/95f8cf8c7594262bf9a4b699b2920115178e4089))
* **rpc:** reexport jsonrpsee::..::ErrorObjectOwned ([#145](https://github.com/fluencelabs/capacity-commitment-prover/issues/145)) ([5a0ed57](https://github.com/fluencelabs/capacity-commitment-prover/commit/5a0ed5757c45cad4f2beea49e6883b3d45cdce73))

## [0.10.2](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.10.1...ccp-v0.10.2) (2024-05-31)


### Bug Fixes

* **test-utils:** enable publishing ([#138](https://github.com/fluencelabs/capacity-commitment-prover/issues/138)) ([9828e57](https://github.com/fluencelabs/capacity-commitment-prover/commit/9828e57f0cd62cdd9d39a67908f024d24c3ca2de))

## [0.10.1](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.10.0...ccp-v0.10.1) (2024-05-30)


### Features

* **cpu-utils:** introduced new abstraction for pinning and topology fetching ([#134](https://github.com/fluencelabs/capacity-commitment-prover/issues/134)) ([696720d](https://github.com/fluencelabs/capacity-commitment-prover/commit/696720da12f559fe2e43ebb7339c0c5878ad89e0))

## [0.10.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.9.0...ccp-v0.10.0) (2024-05-24)


### ⚠ BREAKING CHANGES

* **ccp:** ccp splitting into async parts ([#130](https://github.com/fluencelabs/capacity-commitment-prover/issues/130))

### Features

* **ccp:** ccp splitting into async parts ([#130](https://github.com/fluencelabs/capacity-commitment-prover/issues/130)) ([955c634](https://github.com/fluencelabs/capacity-commitment-prover/commit/955c63428ee8fba6729a349220c6007d07f31ee2))

  It make RPC API more responsive, and prevents 2-minutes delay on `ccp` restart.

  This is a breaking change for `ccp` libraries, but the binary's arguments, serialization formats and RPC interfaces stay the same.



## [0.9.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.8.0...ccp-v0.9.0) (2024-04-17)


### ⚠ BREAKING CHANGES

* **ccp:** fine grain dataset generation ([#125](https://github.com/fluencelabs/capacity-commitment-prover/issues/125))

### Features

* **ccp:** fine grain dataset generation ([#125](https://github.com/fluencelabs/capacity-commitment-prover/issues/125)) ([2595e81](https://github.com/fluencelabs/capacity-commitment-prover/commit/2595e81e284b73c50ae5cfc1cbb6e1f6a926a9a7))
* **ccp:** split UtilityThread functionality in two: StatsCollecting and StateOperating [fixes VM-589] ([#129](https://github.com/fluencelabs/capacity-commitment-prover/issues/129)) ([fc1ca55](https://github.com/fluencelabs/capacity-commitment-prover/commit/fc1ca55632bf48933c3244fdd460b6eb032d0631))
* **rpc:** new CCPRpcClient constructors ([#120](https://github.com/fluencelabs/capacity-commitment-prover/issues/120)) ([ba685ae](https://github.com/fluencelabs/capacity-commitment-prover/commit/ba685ae1b2a189aa945d37576f02dc7d6eab41cf))
* **shared:** implement FromStr for shared types ([#121](https://github.com/fluencelabs/capacity-commitment-prover/issues/121)) ([001f8bc](https://github.com/fluencelabs/capacity-commitment-prover/commit/001f8bceb1cf25df4a06882ee7476e7d1cfcfdbf))
* **shared:** readable Debug for binary types ([#115](https://github.com/fluencelabs/capacity-commitment-prover/issues/115)) ([2553a77](https://github.com/fluencelabs/capacity-commitment-prover/commit/2553a772f99ae6c4423e1a22b0c61d19d739d22d))
* **utility-thread:** SyncProvingThread now explicitly notifies UtilityThread with the current epoch building the cache up [fixes VM-594] ([#128](https://github.com/fluencelabs/capacity-commitment-prover/issues/128)) ([fb61ccd](https://github.com/fluencelabs/capacity-commitment-prover/commit/fb61ccd920feed9211bca8d0f618275100c10ccd))


### Bug Fixes

* **ccp:** manage state in the utility thread ([#127](https://github.com/fluencelabs/capacity-commitment-prover/issues/127)) ([dc5e3c4](https://github.com/fluencelabs/capacity-commitment-prover/commit/dc5e3c47b5ea3810c82d48b33eb462fc7f0ce543))
* **main:** do not try to pin to an empty CPU set ([#126](https://github.com/fluencelabs/capacity-commitment-prover/issues/126)) ([4106bd7](https://github.com/fluencelabs/capacity-commitment-prover/commit/4106bd7b962079aebcc657a41f4ac80e25e9c352))
* **rpc:** make pub use ClientError ([#131](https://github.com/fluencelabs/capacity-commitment-prover/issues/131)) ([270a51d](https://github.com/fluencelabs/capacity-commitment-prover/commit/270a51de9ba388ae9fa43404d1aaa0790e241836))

## [0.8.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.7.1...ccp-v0.8.0) (2024-03-17)


### ⚠ BREAKING CHANGES

* **config,ccp:** add more config options ([#99](https://github.com/fluencelabs/capacity-commitment-prover/issues/99))

### Features

* **config,ccp:** add more config options ([#99](https://github.com/fluencelabs/capacity-commitment-prover/issues/99)) ([0188711](https://github.com/fluencelabs/capacity-commitment-prover/commit/0188711cc7476a4cccf47af68cc64745c3bca9a1))


### Bug Fixes

* CUProver wasn't setting correct pinned core after repinning ([#114](https://github.com/fluencelabs/capacity-commitment-prover/issues/114)) ([ee56885](https://github.com/fluencelabs/capacity-commitment-prover/commit/ee568852d2df3c705d5583aaa454ce264a59b3cc))
* **hashrate:** fix instant hashrate computation ([#111](https://github.com/fluencelabs/capacity-commitment-prover/issues/111)) ([36efb58](https://github.com/fluencelabs/capacity-commitment-prover/commit/36efb58f255640605516263bd748f3d33a303386))

## [0.7.1](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.7.0...ccp-v0.7.1) (2024-03-12)


### Bug Fixes

* **cargo:** move ccp-test-utils dependency to dev-dependencies of ccp ([#108](https://github.com/fluencelabs/capacity-commitment-prover/issues/108)) ([40d4205](https://github.com/fluencelabs/capacity-commitment-prover/commit/40d420526dcb02c0a9f94dffc2bb99264fcd71df))

## [0.7.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.6.0...ccp-v0.7.0) (2024-03-12)


### ⚠ BREAKING CHANGES

* **config:** allow empty initial utility cores set ([#98](https://github.com/fluencelabs/capacity-commitment-prover/issues/98))

### Features

* **config:** allow empty initial utility cores set ([#98](https://github.com/fluencelabs/capacity-commitment-prover/issues/98)) ([e5b73f8](https://github.com/fluencelabs/capacity-commitment-prover/commit/e5b73f8fdc44f5c63094d2e118f29a2c0f8f3570))

## [0.6.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.5.0...ccp-v0.6.0) (2024-03-12)


### ⚠ BREAKING CHANGES

* **rpc:** more ergonomic RPC client ([#102](https://github.com/fluencelabs/capacity-commitment-prover/issues/102))

### Features

* **rpc:** more ergonomic RPC client ([#102](https://github.com/fluencelabs/capacity-commitment-prover/issues/102)) ([0146973](https://github.com/fluencelabs/capacity-commitment-prover/commit/01469732b22996e49c18a9d6fa63acf9aeee2c1c))

## [0.5.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.4.2...ccp-v0.5.0) (2024-03-11)


### ⚠ BREAKING CHANGES

* **cfg,ccp:** store utility threads pinning to saved state ([#87](https://github.com/fluencelabs/capacity-commitment-prover/issues/87))

### Features

* **cfg,ccp:** store utility threads pinning to saved state ([#87](https://github.com/fluencelabs/capacity-commitment-prover/issues/87)) ([1b2ccd5](https://github.com/fluencelabs/capacity-commitment-prover/commit/1b2ccd5982f59783e06f12f5e037b40f3d0c9c0c))

## [0.4.2](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.4.1...ccp-v0.4.2) (2024-03-10)


### Bug Fixes

* stop prover threads simultaneously ([#93](https://github.com/fluencelabs/capacity-commitment-prover/issues/93)) ([dc183fd](https://github.com/fluencelabs/capacity-commitment-prover/commit/dc183fd4eeb12a3870bed27f319fa5882fc57854))

## [0.4.1](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.4.0...ccp-v0.4.1) (2024-03-09)


### Features

* **ccp-shared:** add Ord to CUID type ([#90](https://github.com/fluencelabs/capacity-commitment-prover/issues/90)) ([2beb3b0](https://github.com/fluencelabs/capacity-commitment-prover/commit/2beb3b065c58e0e792a742e3378a685b275b53e9))


### Bug Fixes

* **config:** default config for logs and state ([#92](https://github.com/fluencelabs/capacity-commitment-prover/issues/92)) ([7f062fc](https://github.com/fluencelabs/capacity-commitment-prover/commit/7f062fc528dea52b0a502d6d7b6fbfb89e773620))

## [0.4.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.3.0...ccp-v0.4.0) (2024-03-08)


### ⚠ BREAKING CHANGES

* **msr, persistence:** refactor msr ([#79](https://github.com/fluencelabs/capacity-commitment-prover/issues/79))

### Features

* **msr, persistence:** refactor msr ([#79](https://github.com/fluencelabs/capacity-commitment-prover/issues/79)) ([8510ac7](https://github.com/fluencelabs/capacity-commitment-prover/commit/8510ac78bb63f5959832c8e1bcb8047f8a4be2c6))


### Bug Fixes

* **config:** fix default config field names, add state dir defaults, create state dir ([#89](https://github.com/fluencelabs/capacity-commitment-prover/issues/89)) ([2684980](https://github.com/fluencelabs/capacity-commitment-prover/commit/2684980ec691c5ee278f1317075bb22d2bc5f057))

## [0.3.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.2.0...ccp-v0.3.0) (2024-03-08)


### ⚠ BREAKING CHANGES

* **rpc:** add realloc_utility_cores RPC method ([#69](https://github.com/fluencelabs/capacity-commitment-prover/issues/69))
* **ccp,main:** Graceful shutdown ([#77](https://github.com/fluencelabs/capacity-commitment-prover/issues/77))
* **core:** make crossterm optional ([#71](https://github.com/fluencelabs/capacity-commitment-prover/issues/71))

### Features

* **ccp,main:** Graceful shutdown ([#77](https://github.com/fluencelabs/capacity-commitment-prover/issues/77)) ([6b2c6f8](https://github.com/fluencelabs/capacity-commitment-prover/commit/6b2c6f85819ad44d70560a95181b68bdf1323eb5))
* **core:** make crossterm optional ([#71](https://github.com/fluencelabs/capacity-commitment-prover/issues/71)) ([b8d18fc](https://github.com/fluencelabs/capacity-commitment-prover/commit/b8d18fcc7dc82e72c9e7cfacc1ac41e14e95f4a5))
* **rpc:** add realloc_utility_cores RPC method ([#69](https://github.com/fluencelabs/capacity-commitment-prover/issues/69)) ([a35fc11](https://github.com/fluencelabs/capacity-commitment-prover/commit/a35fc11e205cda2d6cd36b871b760dcac7ddf666))


### Bug Fixes

* **ccp:** use random temporary file name ([#78](https://github.com/fluencelabs/capacity-commitment-prover/issues/78)) ([f8dd6b4](https://github.com/fluencelabs/capacity-commitment-prover/commit/f8dd6b458033476ced2fb00b3ca59c6310e88998))
* **rpc:** get_proofs_after returns empty list on busy lock ([#81](https://github.com/fluencelabs/capacity-commitment-prover/issues/81)) ([84b8e95](https://github.com/fluencelabs/capacity-commitment-prover/commit/84b8e956dca1e946d35598605c2c27330a2ceb54))

## [0.2.0](https://github.com/fluencelabs/capacity-commitment-prover/compare/ccp-v0.1.0...ccp-v0.2.0) (2024-03-06)


### ⚠ BREAKING CHANGES

* **cfg:** Rename [http-server] to [rpc-endpoint] ([#56](https://github.com/fluencelabs/capacity-commitment-prover/issues/56))
* **core:** make all runnables pausable ([#25](https://github.com/fluencelabs/capacity-commitment-prover/issues/25))
* **api:** use EpochParameters where possible ([#24](https://github.com/fluencelabs/capacity-commitment-prover/issues/24))
* **rpc:** add limit to `get_proofs_after` method ([#23](https://github.com/fluencelabs/capacity-commitment-prover/issues/23))

### Features

* **api:** use EpochParameters where possible ([#24](https://github.com/fluencelabs/capacity-commitment-prover/issues/24)) ([33af209](https://github.com/fluencelabs/capacity-commitment-prover/commit/33af209d657e52f0bb8cdc920eff39a7c03df225))
* background on_commitment handling ([#51](https://github.com/fluencelabs/capacity-commitment-prover/issues/51)) ([d9deedf](https://github.com/fluencelabs/capacity-commitment-prover/commit/d9deedfc6fb8c4a0a85db94f93a7f349a24043a7))
* **ccp:** persistent state ([#43](https://github.com/fluencelabs/capacity-commitment-prover/issues/43)) ([d864c89](https://github.com/fluencelabs/capacity-commitment-prover/commit/d864c89e438f671a8722b9272dba258befbc4b93))
* **cfg:** make `[optimizations]` optional ([#61](https://github.com/fluencelabs/capacity-commitment-prover/issues/61)) ([6604f71](https://github.com/fluencelabs/capacity-commitment-prover/commit/6604f71ebc057653d723e8100f68495e87365145))
* **cli:** CLI checks that directories exist ([#14](https://github.com/fluencelabs/capacity-commitment-prover/issues/14)) ([5c5a69c](https://github.com/fluencelabs/capacity-commitment-prover/commit/5c5a69c59e18ebbd0f3c5146a784e640c0ae457f))
* collect hashrate ([#44](https://github.com/fluencelabs/capacity-commitment-prover/issues/44)) ([74da415](https://github.com/fluencelabs/capacity-commitment-prover/commit/74da41560e96d34c24c127a867506859e42db5c6))
* **core:** add result hash into CCProof ([#22](https://github.com/fluencelabs/capacity-commitment-prover/issues/22)) ([1e61762](https://github.com/fluencelabs/capacity-commitment-prover/commit/1e6176228017e626cdd09ddbc7659cd64d3cdf29))
* **core:** make all runnables pausable ([#25](https://github.com/fluencelabs/capacity-commitment-prover/issues/25)) ([04aa11b](https://github.com/fluencelabs/capacity-commitment-prover/commit/04aa11bf0b78dd82086fcac10e8ff011d8abd055))
* **core:** propate errors, introduce utility thread ([#31](https://github.com/fluencelabs/capacity-commitment-prover/issues/31)) ([d45e762](https://github.com/fluencelabs/capacity-commitment-prover/commit/d45e7626f62e8034964265d301c59b135a3c0a13))
* **cpu-utils:** expose physical_cores ([17479d1](https://github.com/fluencelabs/capacity-commitment-prover/commit/17479d142e6c7344ff3f0e60cd9e6628c3fd5961))
* implement thread pinning ([#11](https://github.com/fluencelabs/capacity-commitment-prover/issues/11)) ([9dd9eff](https://github.com/fluencelabs/capacity-commitment-prover/commit/9dd9effbb3a61ca08da78ca65175077b9b67728d))
* introduce configs ([#54](https://github.com/fluencelabs/capacity-commitment-prover/issues/54)) ([de3eea2](https://github.com/fluencelabs/capacity-commitment-prover/commit/de3eea274191b5ae6f6a1ad0415d8d758f25f992))
* **optimization:** CPU cache control to enable hashrate optimizations [fixes VM-488] ([5de0546](https://github.com/fluencelabs/capacity-commitment-prover/commit/5de05462da4444f7fd3633fecaecd82c854a1618))
* optional prometheus endpoint ([#55](https://github.com/fluencelabs/capacity-commitment-prover/issues/55)) ([539cc37](https://github.com/fluencelabs/capacity-commitment-prover/commit/539cc3794ea175d96aa5e64599ed113b202c1577))
* refactor ProvingThread ([#10](https://github.com/fluencelabs/capacity-commitment-prover/issues/10)) ([7751e6e](https://github.com/fluencelabs/capacity-commitment-prover/commit/7751e6e488d158cc8ac09e57bcedc1b7e7f7ae5c))
* **rpc:** add limit to `get_proofs_after` method ([#23](https://github.com/fluencelabs/capacity-commitment-prover/issues/23)) ([86b5579](https://github.com/fluencelabs/capacity-commitment-prover/commit/86b55795bfb1bdea85cd312606eaadd0b2a4cfcd))
* **rpc:** OrHex type uses FromHex and unhex method ([#15](https://github.com/fluencelabs/capacity-commitment-prover/issues/15)) ([c477aec](https://github.com/fluencelabs/capacity-commitment-prover/commit/c477aecd78cc580593bbfc8e30e3b41e6959b44d))
* **shared:** implement Display for certain types ([#63](https://github.com/fluencelabs/capacity-commitment-prover/issues/63)) ([dccf7e5](https://github.com/fluencelabs/capacity-commitment-prover/commit/dccf7e5cdba05ae77f752592e2be9da65277aa5c))
* **shared:** implement ToHex for various types ([#28](https://github.com/fluencelabs/capacity-commitment-prover/issues/28)) ([f82d08b](https://github.com/fluencelabs/capacity-commitment-prover/commit/f82d08b0a1dbc77a25b5bd7f2f91976f36561ad6))
* **shared:** opaque types ([#13](https://github.com/fluencelabs/capacity-commitment-prover/issues/13)) ([f3f8e8e](https://github.com/fluencelabs/capacity-commitment-prover/commit/f3f8e8e3720bf8e2c10c91cbd732d873f2a0878d))
* single state dir ([#50](https://github.com/fluencelabs/capacity-commitment-prover/issues/50)) ([87f0d36](https://github.com/fluencelabs/capacity-commitment-prover/commit/87f0d365626552a54f1d2dac8d04b36e352cbc1b))


### Bug Fixes

* **cfg:** Rename [http-server] to [rpc-endpoint] ([#56](https://github.com/fluencelabs/capacity-commitment-prover/issues/56)) ([ca81b4f](https://github.com/fluencelabs/capacity-commitment-prover/commit/ca81b4f16958bff759d9b0f78dbc2724d6cfb419))
* **client:** new doens't need self ([#12](https://github.com/fluencelabs/capacity-commitment-prover/issues/12)) ([49bf269](https://github.com/fluencelabs/capacity-commitment-prover/commit/49bf2698d46cf720587d3430329a6ab1401e3c66))
* **cli:** fix dir detection ([#17](https://github.com/fluencelabs/capacity-commitment-prover/issues/17)) ([3ea8cee](https://github.com/fluencelabs/capacity-commitment-prover/commit/3ea8cee84668dbf141af2e2c4ef269294c44656b))
* **config:** Correct Default for UnresolvedOptimizations ([#62](https://github.com/fluencelabs/capacity-commitment-prover/issues/62)) ([3470f68](https://github.com/fluencelabs/capacity-commitment-prover/commit/3470f68135c72bf22405dd19f46218248d800db8))
* **crates:** Fix typo ([#8](https://github.com/fluencelabs/capacity-commitment-prover/issues/8)) ([f12341c](https://github.com/fluencelabs/capacity-commitment-prover/commit/f12341c01d0a08e3a2de3bf34cec507200cbf19c))
* fix cross compilation to arm ([#57](https://github.com/fluencelabs/capacity-commitment-prover/issues/57)) ([f1c85a0](https://github.com/fluencelabs/capacity-commitment-prover/commit/f1c85a015ec1a2392b1a20a1f0279da79e96313c))
* **msr:** the code now compiles on MacOs ([#41](https://github.com/fluencelabs/capacity-commitment-prover/issues/41)) ([a7241a9](https://github.com/fluencelabs/capacity-commitment-prover/commit/a7241a9bf169ed6687df32a6cf7710df4fe4ba07))
* refine CLI interface and fix README ([#59](https://github.com/fluencelabs/capacity-commitment-prover/issues/59)) ([93027df](https://github.com/fluencelabs/capacity-commitment-prover/commit/93027df80c168058bad0ec0a3ffaf005abb63dc8))
* refine config saving ([#60](https://github.com/fluencelabs/capacity-commitment-prover/issues/60)) ([a21321d](https://github.com/fluencelabs/capacity-commitment-prover/commit/a21321d1251c22dd1cfd51962ccf8fc2d6a181f2))
* remove excess linux attrs ([#58](https://github.com/fluencelabs/capacity-commitment-prover/issues/58)) ([06c7359](https://github.com/fluencelabs/capacity-commitment-prover/commit/06c73595ad2ac740fe4bd8ae65b5afbc941ab2c5))
* **rpc:** add missing get_proofs_after method to client ([#21](https://github.com/fluencelabs/capacity-commitment-prover/issues/21)) ([92de08e](https://github.com/fluencelabs/capacity-commitment-prover/commit/92de08e8c457204bf3ff80354bbedbf3b8e5d7e7))
* **rpc:** Use ProofIdx type in API types ([#20](https://github.com/fluencelabs/capacity-commitment-prover/issues/20)) ([2c365ec](https://github.com/fluencelabs/capacity-commitment-prover/commit/2c365ec658bf58dad861f23af4fb9bdc4188c6ac))
* **shared:** implement Display for Difficulty ([#66](https://github.com/fluencelabs/capacity-commitment-prover/issues/66)) ([0be20a5](https://github.com/fluencelabs/capacity-commitment-prover/commit/0be20a5c461f9191d8e83fdef33d376203565562))
* stop provers while cleaning proof cache up ([#27](https://github.com/fluencelabs/capacity-commitment-prover/issues/27)) ([0dee01c](https://github.com/fluencelabs/capacity-commitment-prover/commit/0dee01c6ad3d06d866b568688e6591afe176f7a6))
