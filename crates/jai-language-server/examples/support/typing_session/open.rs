//! Opening the sessions the checker asks questions of.

use super::{DocumentUri, Input, Limits, Session};

/// A session over the input's environment with its setup files and `texts` open, as an editor
/// opening those documents has it. The checker's own and every fresh one it compares with are
/// made here.
pub(super) fn open_session(input: &Input<'_>, texts: &[String]) -> Result<Session, String> {
    let mut session = Session::with_environment(Limits::default(), (input.environment)());
    // Answers must not depend on how fast the machine is: a check that hit a wall-clock limit
    // is partial, and a fresh session would not be partial in the same places. A runaway
    // `#run` is still bounded by the evaluation budget and the request budget.
    session.set_check_time_limit(0.0);
    for (uri, text) in &input.setup {
        session
            .open(uri.clone(), 1, text.clone())
            .map_err(|e| format!("{e:?}"))?;
    }
    session.set_auto_import(false);
    for ((uri, _), text) in input.docs.iter().zip(texts) {
        session
            .open(uri.clone(), 1, text.clone())
            .map_err(|e| format!("{e:?}"))?;
    }
    Ok(session)
}
