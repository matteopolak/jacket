//! Register after the paired baked callback invocation gate is measured.
use super::check;

#[test]
fn required_baked_callback_use_rechecks_the_same_executable_specialization() {
    let prefix = "Optional::#type()->int;Required::#type()->int #must;answer::()->int{return 21;}";
    for marker in ["$", "$$"] {
        let consumed = format!(
            "{prefix}consume::({marker}callback:$F)->int{{return callback();}}main::()->int{{first:=consume(cast(Optional)answer);return first+consume(cast(Required)answer);}}"
        );
        let library = check(&consumed).unwrap();
        assert_eq!(
            library.procedures().len(),
            3,
            "same target and executable policy: {consumed}"
        );
        let ignored = format!(
            "{prefix}ignore::({marker}callback:$F)->int{{callback();return 21;}}main::()->int{{first:=ignore(cast(Optional)answer);return first+ignore(cast(Required)answer);}}"
        );
        let error = check(&ignored)
            .err()
            .expect("baked callback specialization must recheck");
        assert!(error.message.contains("#must"), "{ignored}\n{error:?}");
        let start = ignored.find("callback()").unwrap();
        assert_eq!(error.location.span.start, start, "{error:?}");
        assert_eq!(
            error.location.span.end,
            start + "callback()".len(),
            "{error:?}"
        );
    }
}
