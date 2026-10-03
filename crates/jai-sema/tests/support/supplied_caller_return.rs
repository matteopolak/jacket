//! Bounded unchanged original macro source, with independently authored dependencies.
pub fn sgpu_source() -> String {
    // Restore the original extra footer separator while the tracked fixture has
    // one EOF newline. Full source/span and byte receipts accompany the fixture.
    let macro_source = concat!(
        include_str!("../fixtures/supplied-sgpu-caller-return.jai"),
        "\n"
    );
    assert!(macro_source.contains("`return Gpu_Result.FATAL_ERROR_UNKNOWN;"));
    format!(
        "DEBUG_ASSERTS::false;VkResult::enum{{SUCCESS::0;ERROR::1;}}Gpu_Result::enum{{SUCCESS::0;FATAL_ERROR_UNKNOWN::1;}}evaluations:int;fallthroughs:int;next::()->VkResult{{evaluations+=1;if evaluations==1 return .ERROR;return .SUCCESS;}}{macro_source}answer::(status:VkResult)->Gpu_Result{{return_if_error(status);fallthroughs+=1;return .SUCCESS;}}main::()->int{{status:=answer(next());success:=answer(next());if status!=.FATAL_ERROR_UNKNOWN || success!=.SUCCESS return 1;if evaluations!=2 || fallthroughs!=1 return 2;return cast(int)status+41;}}"
    )
}
