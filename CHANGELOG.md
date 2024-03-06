# Changelog

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