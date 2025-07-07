use rustyline::{Cmd, ConditionalEventHandler, Event, EventContext, KeyEvent, RepeatCount};

#[derive(Debug, Clone)]
pub struct XodCompleteHintHandler;
impl ConditionalEventHandler for XodCompleteHintHandler {
    fn handle(&self, evt: &Event, _: RepeatCount, _: bool, ctx: &EventContext) -> Option<Cmd> {
        if !ctx.has_hint() {
            return None; // default
        }
        if let Some(k) = evt.get(0) {
            #[allow(clippy::if_same_then_else)]
            if *k == KeyEvent::ctrl('E') {
                Some(Cmd::CompleteHint)
            } else if *k == KeyEvent::alt('f') && ctx.line().len() == ctx.pos() {
                let text = ctx.hint_text()?;
                let mut start = 0;
                if let Some(first) = text.chars().next()
                    && !first.is_alphanumeric()
                {
                    start = text.find(|c: char| c.is_alphanumeric()).unwrap_or_default();
                }

                let text = text
                    .chars()
                    .enumerate()
                    .take_while(|(i, c)| *i <= start || c.is_alphanumeric())
                    .map(|(_, c)| c)
                    .collect::<String>();

                Some(Cmd::Insert(1, text))
            } else {
                None
            }
        } else {
            unreachable!()
        }
    }
}

pub struct XodTabEventHandler;
impl ConditionalEventHandler for XodTabEventHandler {
    fn handle(&self, evt: &Event, n: RepeatCount, _: bool, ctx: &EventContext) -> Option<Cmd> {
        debug_assert_eq!(*evt, Event::from(KeyEvent::from('\t')));
        if ctx.line()[..ctx.pos()]
            .chars()
            .next_back()
            .filter(|c| c.is_whitespace())
            .is_some()
        {
            if n == 1 {
                Some(Cmd::Insert(1, "   ".to_string()))
            } else {
                let mut string = "   ".to_string();
                for _ in 1..n {
                    string.push_str("    ");
                }
                Some(Cmd::Insert(1, string))
            }
        } else {
            None // default complete
        }
    }
}
