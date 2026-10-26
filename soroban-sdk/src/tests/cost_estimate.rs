use crate as soroban_sdk;
use expect_test::expect;
use soroban_sdk::xdr::ContractCostType;
use soroban_sdk::Env;
use soroban_sdk_macros::symbol_short;

mod contract_data {
    use crate as soroban_sdk;
    soroban_sdk::contractimport!(file = "test_wasms/test_contract_data.wasm");
}

// Update the test data in this test via running it with `UPDATE_EXPECT=1`.
#[test]
fn test_cost_estimate_with_storage() {
    let e = Env::default();

    let contract_id = e.register(contract_data::WASM, ());
    let client = contract_data::Client::new(&e, &contract_id);

    // Write a single new entry to the storage.
    client.put(&symbol_short!("k1"), &symbol_short!("v1"));
    expect![[r#"
        InvocationResources {
            instructions: 237703,
            mem_bytes: 1130075,
            disk_read_entries: 0,
            memory_read_entries: 3,
            write_entries: 1,
            disk_read_bytes: 0,
            write_bytes: 80,
            contract_events_size_bytes: 0,
            persistent_rent_ledger_bytes: 327600,
            persistent_entry_rent_bumps: 1,
            temporary_rent_ledger_bytes: 0,
            temporary_entry_rent_bumps: 0,
        }"#]]
    .assert_eq(format!("{:#?}", e.cost_estimate().resources()).as_str());
    expect![[r#"
        FeeEstimate {
            total: 10001,
            instructions: 167,
            disk_read_entries: 1563,
            write_entries: 2500,
            disk_read_bytes: 0,
            write_bytes: 69,
            contract_events: 0,
            persistent_entry_rent: 5702,
            temporary_entry_rent: 0,
        }"#]]
    .assert_eq(format!("{:#?}", e.cost_estimate().fee()).as_str());

    // Read an entry from the storage. Now there are no write-related resources
    // and fees consumed.
    assert_eq!(client.get(&symbol_short!("k1")), Some(symbol_short!("v1")));
    expect![[r#"
        InvocationResources {
            instructions: 235697,
            mem_bytes: 1129124,
            disk_read_entries: 0,
            memory_read_entries: 3,
            write_entries: 0,
            disk_read_bytes: 0,
            write_bytes: 0,
            contract_events_size_bytes: 0,
            persistent_rent_ledger_bytes: 0,
            persistent_entry_rent_bumps: 0,
            temporary_rent_ledger_bytes: 0,
            temporary_entry_rent_bumps: 0,
        }"#]]
    .assert_eq(format!("{:#?}", e.cost_estimate().resources()).as_str());
    expect![[r#"
        FeeEstimate {
            total: 165,
            instructions: 165,
            disk_read_entries: 0,
            write_entries: 0,
            disk_read_bytes: 0,
            write_bytes: 0,
            contract_events: 0,
            persistent_entry_rent: 0,
            temporary_entry_rent: 0,
        }"#]]
    .assert_eq(format!("{:#?}", e.cost_estimate().fee()).as_str());

    // Delete the entry. There is 1 write_entry, but 0 write_bytes and no rent
    // as this is deletion.
    client.del(&symbol_short!("k1"));
    expect![[r#"
        InvocationResources {
            instructions: 233250,
            mem_bytes: 1129312,
            disk_read_entries: 0,
            memory_read_entries: 3,
            write_entries: 1,
            disk_read_bytes: 0,
            write_bytes: 0,
            contract_events_size_bytes: 0,
            persistent_rent_ledger_bytes: 0,
            persistent_entry_rent_bumps: 0,
            temporary_rent_ledger_bytes: 0,
            temporary_entry_rent_bumps: 0,
        }"#]]
    .assert_eq(format!("{:#?}", e.cost_estimate().resources()).as_str());
    expect![[r#"
        FeeEstimate {
            total: 4227,
            instructions: 164,
            disk_read_entries: 1563,
            write_entries: 2500,
            disk_read_bytes: 0,
            write_bytes: 0,
            contract_events: 0,
            persistent_entry_rent: 0,
            temporary_entry_rent: 0,
        }"#]]
    .assert_eq(format!("{:#?}", e.cost_estimate().fee()).as_str());

    // Read an entry again, now it no longer exists, so there is less read_bytes
    // than in the case when the entry is present.
    assert_eq!(client.get(&symbol_short!("k1")), None);
    expect![[r#"
        InvocationResources {
            instructions: 233517,
            mem_bytes: 1128956,
            disk_read_entries: 0,
            memory_read_entries: 3,
            write_entries: 0,
            disk_read_bytes: 0,
            write_bytes: 0,
            contract_events_size_bytes: 0,
            persistent_rent_ledger_bytes: 0,
            persistent_entry_rent_bumps: 0,
            temporary_rent_ledger_bytes: 0,
            temporary_entry_rent_bumps: 0,
        }"#]]
    .assert_eq(format!("{:#?}", e.cost_estimate().resources()).as_str());
    expect![[r#"
        FeeEstimate {
            total: 164,
            instructions: 164,
            disk_read_entries: 0,
            write_entries: 0,
            disk_read_bytes: 0,
            write_bytes: 0,
            contract_events: 0,
            persistent_entry_rent: 0,
            temporary_entry_rent: 0,
        }"#]]
    .assert_eq(format!("{:#?}", e.cost_estimate().fee()).as_str());
}

#[test]
fn test_cost_estimate_budget() {
    let e = Env::default();

    let contract_id = e.register(contract_data::WASM, ());
    let client = contract_data::Client::new(&e, &contract_id);

    client.put(&symbol_short!("k1"), &symbol_short!("v1"));

    // Budget breakdown corresponds to the last invocation only.
    expect![[r#"
        ===============================================================================================================================================================================
        Cpu limit: 400000000; used: 252152
        Mem limit: 41943040; used: 1148581
        ===============================================================================================================================================================================
        CostType                           iterations     input          cpu_insns      mem_bytes      const_term_cpu      lin_term_cpu        const_term_mem      lin_term_mem        
        WasmInsnExec                       284            None           1136           0              4                   0                   0                   0                   
        MemAlloc                           22             Some(1051385)  140969         1051737        434                 16                  16                  128                 
        MemCpy                             91             Some(9407)     4983           0              42                  16                  0                   0                   
        MemCmp                             57             Some(1736)     2714           0              44                  16                  0                   0                   
        DispatchHostFunction               1              None           295            0              295                 0                   0                   0                   
        VisitObject                        2              None           120            0              60                  0                   0                   0                   
        ValSer                             61             Some(1248)     13690          18506          221                 26                  242                 384                 
        ValDeser                           0              Some(0)        0              0              331                 4369                0                   384                 
        ComputeSha256Hash                  0              Some(0)        0              0              3636                7013                0                   0                   
        ComputeEd25519PubKey               0              None           0              0              40256               0                   0                   0                   
        VerifyEd25519Sig                   0              Some(0)        0              0              377551              4059                0                   0                   
        VmInstantiation                    0              Some(0)        0              0              417482              45712               132773              4903                
        VmCachedInstantiation              0              Some(0)        0              0              41142               634                 69472               1217                
        InvokeVmFunction                   1              None           1945           14             1945                0                   14                  0                   
        ComputeKeccak256Hash               0              Some(0)        0              0              6481                5943                0                   0                   
        DecodeEcdsaCurve256Sig             0              None           0              0              711                 0                   0                   0                   
        RecoverEcdsaSecp256k1Key           0              None           0              0              2314804             0                   181                 0                   
        Int256AddSub                       0              None           0              0              4176                0                   99                  0                   
        Int256Mul                          0              None           0              0              4716                0                   99                  0                   
        Int256Div                          0              None           0              0              4680                0                   99                  0                   
        Int256Pow                          0              None           0              0              4256                0                   99                  0                   
        Int256Shift                        0              None           0              0              884                 0                   99                  0                   
        ChaCha20DrawBytes                  0              Some(0)        0              0              1059                502                 0                   0                   
        ParseWasmInstructions              0              Some(0)        0              0              73077               25410               17564               6457                
        ParseWasmFunctions                 0              Some(0)        0              0              0                   540752              0                   47464               
        ParseWasmGlobals                   0              Some(0)        0              0              0                   176363              0                   13420               
        ParseWasmTableEntries              0              Some(0)        0              0              0                   29989               0                   6285                
        ParseWasmTypes                     0              Some(0)        0              0              0                   1061449             0                   64670               
        ParseWasmDataSegments              0              Some(0)        0              0              0                   237336              0                   29074               
        ParseWasmElemSegments              0              Some(0)        0              0              0                   328476              0                   48095               
        ParseWasmImports                   0              Some(0)        0              0              0                   701845              0                   103229              
        ParseWasmExports                   0              Some(0)        0              0              0                   429383              0                   36394               
        ParseWasmDataSegmentBytes          0              Some(0)        0              0              0                   28                  0                   257                 
        InstantiateWasmInstructions        1              None           43030          70704          43030               0                   70704               0                   
        InstantiateWasmFunctions           1              Some(5)        295            570            0                   7556                0                   14613               
        InstantiateWasmGlobals             1              Some(3)        251            160            0                   10711               0                   6833                
        InstantiateWasmTableEntries        1              Some(0)        0              0              0                   3300                0                   1025                
        InstantiateWasmTypes               1              None           0              0              0                   0                   0                   0                   
        InstantiateWasmDataSegments        1              Some(3)        539            3038           0                   23038               0                   129632              
        InstantiateWasmElemSegments        1              Some(0)        0              0              0                   42488               0                   13665               
        InstantiateWasmImports             1              Some(4)        25905          3051           0                   828974              0                   97637               
        InstantiateWasmExports             1              Some(7)        16247          501            0                   297100              0                   9176                
        InstantiateWasmDataSegmentBytes    1              Some(305)      33             300            0                   14                  0                   126                 
        Sec1DecodePointUncompressed        0              None           0              0              1882                0                   0                   0                   
        VerifyEcdsaSecp256r1Sig            0              None           0              0              3000906             0                   0                   0                   
        Bls12381EncodeFp                   0              None           0              0              661                 0                   0                   0                   
        Bls12381DecodeFp                   0              None           0              0              985                 0                   0                   0                   
        Bls12381G1CheckPointOnCurve        0              None           0              0              1934                0                   0                   0                   
        Bls12381G1CheckPointInSubgroup     0              None           0              0              730510              0                   0                   0                   
        Bls12381G2CheckPointOnCurve        0              None           0              0              5921                0                   0                   0                   
        Bls12381G2CheckPointInSubgroup     0              None           0              0              1057822             0                   0                   0                   
        Bls12381G1ProjectiveToAffine       0              None           0              0              92642               0                   0                   0                   
        Bls12381G2ProjectiveToAffine       0              None           0              0              100742              0                   0                   0                   
        Bls12381G1Add                      0              None           0              0              7689                0                   0                   0                   
        Bls12381G1Mul                      0              None           0              0              2458985             0                   0                   0                   
        Bls12381G1Msm                      0              Some(0)        0              0              2347584             94135478            109494              266603              
        Bls12381MapFpToG1                  0              None           0              0              1020885             0                   2776                0                   
        Bls12381HashToG1                   0              Some(0)        0              0              2638451             6803                5896                0                   
        Bls12381G2Add                      0              None           0              0              25207               0                   0                   0                   
        Bls12381G2Mul                      0              None           0              0              7873219             0                   0                   0                   
        Bls12381G2Msm                      0              Some(0)        0              0              7663880             298580871           219654              266603              
        Bls12381MapFp2ToG2                 0              None           0              0              1856539             0                   1672                0                   
        Bls12381HashToG2                   0              Some(0)        0              0              6315452             7232                3960                0                   
        Bls12381Pairing                    0              Some(0)        0              0              10558948            632860943           2204                9340474             
        Bls12381FrFromU256                 0              None           0              0              1994                0                   0                   0                   
        Bls12381FrToU256                   0              None           0              0              1155                0                   248                 0                   
        Bls12381FrAddSub                   0              None           0              0              74                  0                   0                   0                   
        Bls12381FrMul                      0              None           0              0              332                 0                   0                   0                   
        Bls12381FrPow                      0              Some(0)        0              0              691                 74558               0                   128                 
        Bls12381FrInv                      0              None           0              0              35421               0                   0                   0                   
        Bn254EncodeFp                      0              None           0              0              344                 0                   0                   0                   
        Bn254DecodeFp                      0              None           0              0              476                 0                   0                   0                   
        Bn254G1CheckPointOnCurve           0              None           0              0              904                 0                   0                   0                   
        Bn254G2CheckPointOnCurve           0              None           0              0              2811                0                   0                   0                   
        Bn254G2CheckPointInSubgroup        0              None           0              0              1706052             0                   0                   0                   
        Bn254G1ProjectiveToAffine          0              None           0              0              61                  0                   0                   0                   
        Bn254G1Add                         0              None           0              0              3623                0                   0                   0                   
        Bn254G1Mul                         0              None           0              0              1150435             0                   0                   0                   
        Bn254Pairing                       0              Some(0)        0              0              5263916             392472814           1821                6232546             
        Bn254FrFromU256                    0              None           0              0              2052                0                   0                   0                   
        Bn254FrToU256                      0              None           0              0              1133                0                   312                 0                   
        Bn254FrAddSub                      0              None           0              0              74                  0                   0                   0                   
        Bn254FrMul                         0              None           0              0              332                 0                   0                   0                   
        Bn254FrPow                         0              Some(0)        0              0              755                 68930               0                   0                   
        Bn254FrInv                         0              None           0              0              33151               0                   0                   0                   
        Bn254G1Msm                         0              Some(0)        0              0              1185193             41568084            73061               229779              
        MlDsa44DecodeVerifyingKey          0              None           0              0              849110              0                   24656               0                   
        MlDsa65DecodeVerifyingKey          0              None           0              0              1505271             0                   43088               0                   
        MlDsa87DecodeVerifyingKey          0              None           0              0              2625760             0                   73808               0                   
        MlDsa44DecodeSignature             0              None           0              0              37888               0                   4104                0                   
        MlDsa65DecodeSignature             0              None           0              0              47691               0                   5128                0                   
        MlDsa87DecodeSignature             0              None           0              0              66346               0                   7176                0                   
        VerifyMlDsa44Sig                   0              Some(0)        0              0              685165              6103                0                   0                   
        VerifyMlDsa65Sig                   0              Some(0)        0              0              983619              6103                0                   0                   
        VerifyMlDsa87Sig                   0              Some(0)        0              0              1437210             6102                0                   0                   
        ===============================================================================================================================================================================
        Internal details (diagnostics info, does not affect fees) 
        Total # times meter was called: 247
        Shadow cpu limit: 400000000; used: 31841
        Shadow mem limit: 41943040; used: 27684
        ===============================================================================================================================================================================


    "#]]
    .assert_eq(format!("{:#?}", e.cost_estimate().budget()).as_str());

    // Data for a specific cost type (one row in the budget table above) can be
    // obtained as well via `tracker` function.
    expect![[r#"
        CostTracker {
            iterations: 284,
            inputs: None,
            cpu: 1136,
            mem: 0,
        }"#]]
    .assert_eq(
        format!(
            "{:#?}",
            e.cost_estimate()
                .budget()
                .tracker(ContractCostType::WasmInsnExec)
        )
        .as_str(),
    );
}

fn extract_mainnet_limits_and_fees(
    upgrade_set: &soroban_sdk::xdr::ConfigUpgradeSet,
) -> (
    soroban_env_host::InvocationResourceLimits,
    soroban_env_host::fees::FeeConfiguration,
    i64,
    i64,
    Option<i64>,
) {
    use soroban_env_host::fees::{
        compute_rent_write_fee_per_1kb, FeeConfiguration, RentWriteFeeConfiguration,
    };
    use soroban_env_host::InvocationResourceLimits;
    use soroban_sdk::xdr::ConfigSettingEntry;

    let mut compute = None;
    let mut ledger_cost = None;
    let mut ledger_cost_ext = None;
    let mut historical_data = None;
    let mut events = None;
    let mut bandwidth = None;
    let mut state_archival = None;
    let mut max_key_size = None;
    let mut max_entry_size = None;
    let mut max_code_size = None;
    let mut state_size_window = None;

    for entry in upgrade_set.updated_entry.iter() {
        match entry {
            ConfigSettingEntry::ContractComputeV0(v) => compute = Some(v.clone()),
            ConfigSettingEntry::ContractLedgerCostV0(v) => ledger_cost = Some(v.clone()),
            ConfigSettingEntry::ContractLedgerCostExtV0(v) => ledger_cost_ext = Some(v.clone()),
            ConfigSettingEntry::ContractHistoricalDataV0(v) => historical_data = Some(v.clone()),
            ConfigSettingEntry::ContractEventsV0(v) => events = Some(v.clone()),
            ConfigSettingEntry::ContractBandwidthV0(v) => bandwidth = Some(v.clone()),
            ConfigSettingEntry::StateArchival(v) => state_archival = Some(v.clone()),
            ConfigSettingEntry::ContractDataKeySizeBytes(v) => max_key_size = Some(*v),
            ConfigSettingEntry::ContractDataEntrySizeBytes(v) => max_entry_size = Some(*v),
            ConfigSettingEntry::ContractMaxSizeBytes(v) => max_code_size = Some(*v),
            ConfigSettingEntry::LiveSorobanStateSizeWindow(v) => {
                state_size_window = Some(v.clone())
            }
            _ => {}
        }
    }

    let compute = compute.expect("missing ContractComputeV0");
    let ledger_cost = ledger_cost.expect("missing ContractLedgerCostV0");
    let ledger_cost_ext = ledger_cost_ext.expect("missing ContractLedgerCostExtV0");
    let historical_data = historical_data.expect("missing ContractHistoricalDataV0");
    let events = events.expect("missing ContractEventsV0");
    let bandwidth = bandwidth.expect("missing ContractBandwidthV0");
    let state_archival = state_archival.expect("missing StateArchival");

    let limits = InvocationResourceLimits {
        instructions: compute.tx_max_instructions,
        mem_bytes: i64::from(compute.tx_memory_limit),
        disk_read_entries: ledger_cost.tx_max_disk_read_entries,
        write_entries: ledger_cost.tx_max_write_ledger_entries,
        ledger_entries: ledger_cost_ext.tx_max_footprint_entries,
        disk_read_bytes: ledger_cost.tx_max_disk_read_bytes,
        write_bytes: ledger_cost.tx_max_write_bytes,
        contract_events_size_bytes: events.tx_max_contract_events_size_bytes,
        max_contract_data_key_size_bytes: max_key_size.expect("missing ContractDataKeySizeBytes"),
        max_contract_data_entry_size_bytes: max_entry_size
            .expect("missing ContractDataEntrySizeBytes"),
        max_contract_code_entry_size_bytes: max_code_size.expect("missing ContractMaxSizeBytes"),
    };

    let fee_config = FeeConfiguration {
        fee_per_instruction_increment: compute.fee_rate_per_instructions_increment,
        fee_per_disk_read_entry: ledger_cost.fee_disk_read_ledger_entry,
        fee_per_write_entry: ledger_cost.fee_write_ledger_entry,
        fee_per_disk_read_1kb: ledger_cost.fee_disk_read1_kb,
        fee_per_write_1kb: ledger_cost_ext.fee_write1_kb,
        fee_per_historical_1kb: historical_data.fee_historical1_kb,
        fee_per_contract_event_1kb: events.fee_contract_events1_kb,
        fee_per_transaction_size_1kb: bandwidth.fee_tx_size1_kb,
    };

    let live_rent_fee_per_1kb = state_size_window.and_then(|window| {
        if window.is_empty() {
            return None;
        }
        let avg_size: u64 =
            window.iter().fold(0u64, |acc, x| acc.saturating_add(*x)) / window.len() as u64;
        let write_fee_config = RentWriteFeeConfiguration {
            state_target_size_bytes: ledger_cost.soroban_state_target_size_bytes,
            rent_fee_1kb_state_size_low: ledger_cost.rent_fee1_kb_soroban_state_size_low,
            rent_fee_1kb_state_size_high: ledger_cost.rent_fee1_kb_soroban_state_size_high,
            state_size_rent_fee_growth_factor: ledger_cost.soroban_state_rent_fee_growth_factor,
        };
        Some(compute_rent_write_fee_per_1kb(
            i64::try_from(avg_size).unwrap(),
            &write_fee_config,
        ))
    });

    (
        limits,
        fee_config,
        state_archival.persistent_rent_rate_denominator,
        state_archival.temp_rent_rate_denominator,
        live_rent_fee_per_1kb,
    )
}

#[test]
fn test_mainnet_limits_and_fees_snapshot() {
    use crate::testutils::cost_estimate::{CostEstimate, NetworkInvocationResourceLimits};
    use soroban_env_host::InvocationResourceLimits;
    use soroban_sdk::xdr::{
        ConfigSettingContractBandwidthV0, ConfigSettingContractComputeV0,
        ConfigSettingContractEventsV0, ConfigSettingContractHistoricalDataV0,
        ConfigSettingContractLedgerCostExtV0, ConfigSettingContractLedgerCostV0,
        ConfigSettingEntry, ConfigUpgradeSet, Limits, ReadXdr, StateArchivalSettings,
    };

    let sdk_limits = InvocationResourceLimits::mainnet();
    let (sdk_fee_config, sdk_fee_per_rent_1kb, sdk_persistent_rent_denom, sdk_temp_rent_denom) =
        CostEstimate::mainnet_fee_config();

    // Ensure the default test budget limits and `InvocationResourceLimits::mainnet()`
    // stay in sync.
    let e = Env::default();
    let budget_debug = format!("{:#?}", e.cost_estimate().budget());
    assert!(
        budget_debug.contains(&format!("Cpu limit: {};", sdk_limits.instructions)),
        "Env::default() budget CPU limit drifted from InvocationResourceLimits::mainnet().instructions"
    );
    assert!(
        budget_debug.contains(&format!("Mem limit: {};", sdk_limits.mem_bytes)),
        "Env::default() budget Mem limit drifted from InvocationResourceLimits::mainnet().mem_bytes"
    );

    // When `MAINNET_NETWORK_SETTINGS_XDR` is provided (e.g. from
    // `stellar network settings --network mainnet --internal --output xdr` in
    // the scheduled drift-detection workflow), compare the live network
    // settings against the hardcoded SDK snapshot. Otherwise, verify the
    // extractor against the 2026-07-10 snapshot fixture.
    let upgrade_set = match std::env::var("MAINNET_NETWORK_SETTINGS_XDR") {
        Ok(xdr_b64) if !xdr_b64.trim().is_empty() => {
            ConfigUpgradeSet::from_xdr_base64(xdr_b64.trim(), Limits::none())
                .expect("failed to decode MAINNET_NETWORK_SETTINGS_XDR as ConfigUpgradeSet")
        }
        _ => ConfigUpgradeSet {
            updated_entry: vec![
                ConfigSettingEntry::ContractMaxSizeBytes(
                    sdk_limits.max_contract_code_entry_size_bytes,
                ),
                ConfigSettingEntry::ContractComputeV0(ConfigSettingContractComputeV0 {
                    ledger_max_instructions: 1_000_000_000,
                    tx_max_instructions: sdk_limits.instructions,
                    fee_rate_per_instructions_increment: sdk_fee_config
                        .fee_per_instruction_increment,
                    tx_memory_limit: u32::try_from(sdk_limits.mem_bytes).unwrap(),
                }),
                ConfigSettingEntry::ContractLedgerCostV0(ConfigSettingContractLedgerCostV0 {
                    ledger_max_disk_read_entries: 1000,
                    ledger_max_disk_read_bytes: 200_000,
                    ledger_max_write_ledger_entries: 500,
                    ledger_max_write_bytes: 132_096,
                    tx_max_disk_read_entries: sdk_limits.disk_read_entries,
                    tx_max_disk_read_bytes: sdk_limits.disk_read_bytes,
                    tx_max_write_ledger_entries: sdk_limits.write_entries,
                    tx_max_write_bytes: sdk_limits.write_bytes,
                    fee_disk_read_ledger_entry: sdk_fee_config.fee_per_disk_read_entry,
                    fee_write_ledger_entry: sdk_fee_config.fee_per_write_entry,
                    fee_disk_read1_kb: sdk_fee_config.fee_per_disk_read_1kb,
                    soroban_state_target_size_bytes: 30_000_000_000,
                    rent_fee1_kb_soroban_state_size_low: 1_000,
                    rent_fee1_kb_soroban_state_size_high: 10_000,
                    soroban_state_rent_fee_growth_factor: 1_000,
                }),
                ConfigSettingEntry::ContractLedgerCostExtV0(ConfigSettingContractLedgerCostExtV0 {
                    tx_max_footprint_entries: sdk_limits.ledger_entries,
                    fee_write1_kb: sdk_fee_config.fee_per_write_1kb,
                }),
                ConfigSettingEntry::ContractHistoricalDataV0(
                    ConfigSettingContractHistoricalDataV0 {
                        fee_historical1_kb: sdk_fee_config.fee_per_historical_1kb,
                    },
                ),
                ConfigSettingEntry::ContractEventsV0(ConfigSettingContractEventsV0 {
                    tx_max_contract_events_size_bytes: sdk_limits.contract_events_size_bytes,
                    fee_contract_events1_kb: sdk_fee_config.fee_per_contract_event_1kb,
                }),
                ConfigSettingEntry::ContractBandwidthV0(ConfigSettingContractBandwidthV0 {
                    ledger_max_txs_size_bytes: 1_000_000,
                    tx_max_size_bytes: 131_072,
                    fee_tx_size1_kb: sdk_fee_config.fee_per_transaction_size_1kb,
                }),
                ConfigSettingEntry::ContractDataKeySizeBytes(
                    sdk_limits.max_contract_data_key_size_bytes,
                ),
                ConfigSettingEntry::ContractDataEntrySizeBytes(
                    sdk_limits.max_contract_data_entry_size_bytes,
                ),
                ConfigSettingEntry::StateArchival(StateArchivalSettings {
                    max_entry_ttl: 6_312_000,
                    min_temporary_ttl: 16,
                    min_persistent_ttl: 2_073_600,
                    persistent_rent_rate_denominator: sdk_persistent_rent_denom,
                    temp_rent_rate_denominator: sdk_temp_rent_denom,
                    max_entries_to_archive: 1000,
                    live_soroban_state_size_window_sample_size: 30,
                    live_soroban_state_size_window_sample_period: 64,
                    eviction_scan_size: 100_000,
                    starting_eviction_scan_level: 7,
                }),
            ]
            .try_into()
            .unwrap(),
        },
    };

    let (
        network_limits,
        network_fee_config,
        network_persistent_rent_denom,
        network_temp_rent_denom,
        live_rent_fee_per_1kb,
    ) = extract_mainnet_limits_and_fees(&upgrade_set);

    assert_eq!(
        network_limits, sdk_limits,
        "InvocationResourceLimits::mainnet() drifted from Mainnet network settings"
    );
    assert_eq!(
        network_fee_config, sdk_fee_config,
        "CostEstimate::fee() FeeConfiguration drifted from Mainnet network settings"
    );
    assert_eq!(
        network_persistent_rent_denom, sdk_persistent_rent_denom,
        "CostEstimate::fee() persistent_rent_rate_denominator drifted from Mainnet network settings"
    );
    assert_eq!(
        network_temp_rent_denom, sdk_temp_rent_denom,
        "CostEstimate::fee() temp_rent_rate_denominator drifted from Mainnet network settings"
    );
    if let Some(live_rent) = live_rent_fee_per_1kb {
        assert!(
            sdk_fee_per_rent_1kb >= live_rent,
            "CostEstimate::fee() conservative fee_per_rent_1kb overestimate ({sdk_fee_per_rent_1kb}) is below live Mainnet rent fee ({live_rent})"
        );
    }
}
