use self::testcase::Testcase;
use orfail::OrFail;
use testcase::Command;

mod testcase;

#[test]
pub fn decode_address() -> orfail::Result<()> {
    let testcase = Testcase::load("address.json").or_fail()?;
    for command in &testcase.commands {
        let Command::Module(command) = command else {
            continue;
        };
        command.decode_module().or_fail()?;
    }
    Ok(())
}
