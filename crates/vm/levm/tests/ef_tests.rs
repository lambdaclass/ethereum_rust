mod ef_tests_executor;
use ef_tests_executor::test_utils::run_test;

use std::{collections::HashSet, path::Path};

use datatest_stable;

fn get_group_name_from_path(path: &Path) -> String {
    // Gets the parent directory's name.
    // Example: ethtests/GeneralStateTests/stArgsZeroOneBalance/addmodNonConst.json
    // -> stArgsZeroOneBalance
    path.ancestors()
        .into_iter()
        .nth(1)
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

fn get_suite_name_from_path(path: &Path) -> String {
    // Example: ethtests/GeneralStateTests/stArgsZeroOneBalance/addmodNonConst.json
    // -> addmodNonConst
    path.file_stem().unwrap().to_str().unwrap().to_string()
}

fn get_ignored_groups() -> HashSet<String> {
    HashSet::from([
        "stEIP1153-transientStorage".into(),
        "eip3651_warm_coinbase".into(),
        "stEIP3651-warmcoinbase".into(),
        "stEIP3860-limitmeterinitcode".into(),
        "vmPerformance".into(),
        "stArgsZeroOneBalance".into(),
        "stCallDelegateCodesHomestead".into(),
        "stCodeSizeLimit".into(),
        "stCreate2".into(),
        "stCreateTest".into(),
        "stEIP158Specific".into(),
        "stEIP4844-blobtransactions".into(),
        "stEIP5656-MCOPY".into(),
        "stEIP2930".into(),
        "stRandom".into(),
        "stRandom2".into(),
        "stRecursiveCreate".into(),
        "stRefundTest".into(),
        "stSStoreTest".into(),
        "stStaticFlagEnabled".into(),
        "stZeroCallsRevert".into(),
        "stTimeConsuming".into(), // this will be tested with the time_consuming_test binary
        "stRevertTest".into(),
        "eip3855_push0".into(),
        "eip4844_blobs".into(),
        "stSystemOperationsTest".into(),
        "stReturnDataTest".into(),
        "stHomesteadSpecific".into(),
        "stStackTests".into(),
        "eip5656_mcopy".into(),
        "eip6780_selfdestruct".into(),
        "stCallCreateCallCodeTest".into(),
        "stPreCompiledContracts2".into(),
        "stZeroKnowledge2".into(),
        "stDelegatecallTestHomestead".into(),
        "stEIP150singleCodeGasPrices".into(),
        "stSpecialTest".into(),
        "vmIOandFlowOperations".into(),
        "stEIP150Specific".into(),
        "stExtCodeHash".into(),
        "stCallCodes".into(),
        "stMemoryStressTest".into(),
        "vmTests".into(),
        "stZeroKnowledge".into(),
        "stLogTests".into(),
        "stBugs".into(),
        "stEIP1559".into(),
        "stStaticCall".into(),
        "stMemExpandingEIP150Calls".into(),
        "stTransactionTest".into(),
        "eip3860_initcode".into(),
        "stCodeCopyTest".into(),
        "stPreCompiledContracts".into(),
        "stNonZeroCallsTest".into(),
        "stMemoryTest".into(),
        "stInitCodeTest".into(),
        "stBadOpcode".into(),
        "eip1153_tstore".into(),
        "stSolidityTest".into(),
        "yul".into(),
        "stEIP3607".into(),
        "eip198_modexp_precompile".into(),
        "stZeroCallsTest".into(),
        "stAttackTest".into(),
        "stExample".into(),
        "vmArithmeticTest".into(),
        "stQuadraticComplexityTest".into(),
        "stSelfBalance".into(),
        "stEIP3855-push0".into(),
        "stWalletTest".into(),
        "vmLogTest".into(),
        "stTransitionTest".into(),
    ])
}

// ls -1 | wc -l -> count number of files in dir

// Current not ignored groups:
// - stShift
// 41 tests
// - eip7516_blobgasfee
// 3 tests
// - Pyspecs/frontier/opcodes
// 2 tests
// - eip2930_access_list
// 1 test
// - eip1344_chainid
// 1 test
// - stChainId
// 2 tests
// - vmBitwiseLogicOperation
// 11 tests
// - stCallDelegateCodesCallCodeHomestead
// 58 tests
// - stSLoadTest
// 1 test
// Total: 120 tests

fn get_ignored_suites() -> HashSet<String> {
    HashSet::from([
        "ValueOverflow".into(),      // TODO: parse bigint tx value
        "ValueOverflowParis".into(), // TODO: parse bigint tx value
    ])
}

fn run_ef_test(path: &Path, contents: String) -> datatest_stable::Result<()> {
    let group_name = get_group_name_from_path(path);

    if get_ignored_groups().contains(&group_name) {
        return Ok(());
    }

    let suite_name = get_suite_name_from_path(path);
    if get_ignored_suites().contains(&suite_name) {
        return Ok(());
    }

    run_test(path, contents)
}

#[cfg(not(feature = "skip_ef_tests"))]
datatest_stable::harness!(run_ef_test, "ethtests/GeneralStateTests/", r"^.*/*.json",);
