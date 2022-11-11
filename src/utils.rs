use std::borrow::Cow;

pub fn truncate_str(s: &str, len: usize) -> Cow<'_, str> {
    if s.chars().count() > len {
        let owned = s
            .chars()
            .take(len.saturating_sub(3))
            .chain("...".chars())
            .collect();
        Cow::Owned(owned)
    } else {
        Cow::Borrowed(s)
    }
}
