use self::testcase::Testcase;
use orfail::OrFail;

mod testcase;

#[test]
pub fn decode_address() -> orfail::Result<()> {
    let testcase = Testcase::load("address.json").or_fail()?;
    Ok(())
}
