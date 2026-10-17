//! Phrase banks for the Quarterly Earnings Call.
//!
//! Variants are chosen deterministically from the quarter and the profile
//! seed, so a given quarter always produces the same transcript.

/// Picks a variant from `bank`. Sections use different `salt`s so they do not
/// all advance in step.
pub fn pick<'a>(bank: &[&'a str], n: u64, salt: u64) -> &'a str {
    bank[(n.wrapping_add(salt.wrapping_mul(7919)) % bank.len() as u64) as usize]
}

/// A Comparable's name as it reads within a sentence. Introduced
/// Comparables are described ("Your manager's manager"); their first word is
/// lowercased there. Names entered by the user are left as entered.
pub fn in_sentence(name: &str) -> String {
    const DESCRIPTIVE: [&str; 5] = ["Your", "A", "An", "The", "Someone"];
    match name.split_once(' ') {
        Some((first, rest)) if DESCRIPTIVE.contains(&first) => {
            format!("{} {rest}", first.to_lowercase())
        }
        _ => name.to_owned(),
    }
}

/// The text with its first letter capitalized, to open a sentence.
pub fn opening(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Fills `{key}` placeholders.
pub fn fill(template: &str, values: &[(&str, &str)]) -> String {
    values.iter().fold(template.to_owned(), |s, (k, v)| {
        s.replace(&format!("{{{k}}}"), v)
    })
}

pub const OPERATOR_OPENING: [&str; 3] = [
    "Good day, and welcome to the {quarter} earnings call for {name}. All participants are in \
     listen-only mode. After the prepared remarks, there will be a question-and-answer session. \
     I would now like to turn the call over to {name}, Chief Executive. Please go ahead.",
    "Good morning, and thank you for standing by. Welcome to {name}'s {quarter} earnings \
     conference call. Today's call is being recorded. I will now hand the call to {name}, Chief \
     Executive.",
    "Welcome to the {quarter} results call for {name}. Following prepared remarks, we will open \
     the line for questions from analysts. I will now turn the call over to the Chief Executive, \
     {name}.",
];

pub const GREETING: [&str; 3] = [
    "Thank you, operator, and good morning, everyone. Thank you for joining us.",
    "Thank you, and thanks to everyone for being with us today.",
    "Thank you, operator. Good morning, and thank you all for joining.",
];

/// The quarter's performance when the index rose. Placeholders: {open},
/// {close}, {change}, {pct}, {high}, {low}.
pub const PERFORMANCE_UP: [&str; 3] = [
    "The Personal Standing Index opened the quarter at {open} and closed at {close}, a gain of \
     {change} points, or {pct}. We are pleased with this result, which reflects disciplined \
     execution across the business.",
    "We closed the quarter at {close}, up {change} points from an open of {open}. The index traded \
     between {low} and {high}. These results speak for themselves.",
    "The index advanced {pct} over the quarter, from {open} to {close}. These results reflect the \
     underlying strength of the franchise.",
];

/// The quarter's performance when the index was unchanged. Placeholders:
/// {close}.
pub const PERFORMANCE_FLAT: [&str; 2] = [
    "The Personal Standing Index closed the quarter unchanged at {close}. We view that stability \
     as a strength.",
    "The index was unchanged over the quarter at {close}. We continue to execute against our plan.",
];

/// The quarter's performance when the index fell.
pub const PERFORMANCE_DOWN: [&str; 3] = [
    "The Personal Standing Index opened the quarter at {open} and closed at {close}, a change of \
     {change} points, or {pct}. This was a challenging quarter, and the result reflects \
     conditions across the peer set rather than any change in our underlying fundamentals.",
    "We closed the quarter at {close}, from an open of {open}. The index traded between {low} and \
     {high}. I want to be clear that we see this as a timing issue.",
    "The index ended the quarter at {close}, against an open of {open}. While the headline figure \
     does not reflect the progress we have made, we remain confident in the strategy.",
];

pub const NO_TRADING: &str =
    "No index values were recorded during the quarter. We do not view this as material.";

/// Threshold revisions. Placeholders: {count}, {from}, {to}.
pub const THRESHOLD_REVISED: [&str; 3] = [
    "During the quarter, the Enough Threshold was revised {count}, from {from} to {to}, to reflect \
     current market conditions. We continue to target the threshold, and we expect to reach it.",
    "As you will have seen in our filings, the Enough Threshold moved {count} this quarter, from \
     {from} to {to}. Our commitment to reaching it is unchanged.",
    "The Enough Threshold was adjusted {count} in the period, and now stands at {to}. I would \
     characterize that as a sign of the strength of the market.",
];

/// Placeholders: {to}.
pub const THRESHOLD_UNCHANGED: [&str; 2] = [
    "The Enough Threshold remained at {to} throughout the quarter. It remains our target.",
    "The Enough Threshold was unchanged at {to}. We are focused on closing the gap.",
];

/// Placeholders: {list}.
pub const HEADWINDS: [&str; 3] = [
    "We did face headwinds in a number of areas, most notably {list}.",
    "I won't pretend the quarter was without challenges. Performance in {list} was below our \
     expectations.",
    "Results were affected by softness in {list}, which we are monitoring closely.",
];

/// Placeholders: {first}, {sector}.
pub const INITIATIVES: [&str; 4] = [
    "Looking ahead, our strategic initiatives include a renewed focus on {first} and a broader \
     review of our position in {sector}.",
    "For the coming quarter, we are prioritizing {first}. We are also undertaking a strategic \
     review of {sector}, and we will update you as appropriate.",
    "We have identified {first} as a key area of investment, and we are realigning resources \
     across {sector} accordingly.",
    "Our priorities going forward are clear. First, {first}. Second, a disciplined approach to \
     {sector}. Third, continued attention to the peer set.",
];

/// Placeholders: {ticker}, {name}, {change}.
pub const PEER_GAINER: [&str; 2] = [
    "Across the peer set, {name} ({ticker}) posted a gain of {change} points. We congratulate \
     them, and we are studying their results carefully.",
    "I would note that {name} ({ticker}) gained {change} points in the quarter. We do not \
     comment on individual Comparables.",
];

/// Placeholders: {count}, {names}.
pub const PEER_ADDED: [&str; 2] = [
    "We also welcomed {count} to the peer set: {names}.",
    "The peer set expanded during the quarter with the addition of {names}.",
];

/// Placeholders: {names}.
pub const DISCOVERIES: [&str; 2] = [
    "The Standard Classification was extended during the quarter to include {names}, now \
     reflected in our results.",
    "Newly classified dimensions this quarter included {names}.",
];

pub const HANDOVER: [&str; 2] = [
    "With that, I will turn the call back to the operator for questions.",
    "Operator, we are now ready to take questions.",
];

/// Analysts' firms. Generic and fictional; never a real institution.
pub const FIRMS: [&str; 8] = [
    "Regional Securities",
    "Independent Research Associates",
    "General Brokerage Services",
    "Provincial Capital Markets",
    "Municipal Equity Desk",
    "Consolidated Advisory Group",
    "Continental Research Partners",
    "Second Street Equities",
];

pub const OPERATOR_FIRST_QUESTION: &str =
    "Our first question comes from the line of an analyst at {firm}. Please go ahead.";

pub const OPERATOR_QUESTION: [&str; 3] = [
    "The next question is from {firm}. Your line is open.",
    "We will take our next question from {firm}.",
    "Our next question comes from {firm}. Please go ahead.",
];

pub const OPERATOR_TO_CLOSING: [&str; 2] = [
    "There are no further questions at this time. I would now like to turn the call back to \
     {name} for closing remarks.",
    "That concludes our question-and-answer session. I will turn the call back to {name} for any \
     closing remarks.",
];

/// Placeholders: {dimension}, {leader} (within a sentence), {Leader} (opening one).
pub const Q_HEADWIND: [&str; 3] = [
    "Thanks for taking the question. Could you speak to performance in {dimension}? It looks like \
     {leader} has widened their lead there.",
    "On {dimension}: can you help us understand what is driving the gap with {leader}, and when \
     you expect it to close?",
    "I wanted to ask about {dimension}. {Leader} appears to be well ahead. Is that structural?",
];

/// Placeholders: {to}, {shortfall}.
pub const Q_THRESHOLD: [&str; 3] = [
    "The Enough Threshold now stands at {to}, with a shortfall of {shortfall}. When do you expect \
     to reach it?",
    "Can you give us any guidance on the timeline for reaching the Enough Threshold? The \
     shortfall is currently {shortfall}.",
    "Is {to} still the right target, in your view?",
];

/// Placeholders: {name} (within a sentence), {Name} (opening one), {change}.
pub const Q_PEER: [&str; 2] = [
    "{Name} gained {change} points this quarter. How are you responding to that?",
    "Could you comment on the performance of {name}? A gain of {change} points seems significant.",
];

/// Placeholders: {count}.
pub const Q_POSITIONS: [&str; 2] = [
    "You currently hold {count}. Can you talk about your strategy there?",
    "You hold {count} at present. Do you expect that to change?",
];

pub const Q_SPOTLIGHT: [&str; 2] = [
    "The Spotlight Monitor continues to read zero. Is that a concern?",
    "Can you comment on the Spotlight readings this quarter?",
];

pub const EVASIONS: [&str; 11] = [
    "We don't break that out.",
    "We're very comfortable with where we are.",
    "I'd point you to the long-term trend.",
    "That's something we're looking at very closely.",
    "We believe the fundamentals remain strong.",
    "We'll have more to say about that next quarter.",
    "I'm not going to speculate.",
    "We see that as a timing issue.",
    "I think the market has yet to fully appreciate our position there.",
    "That's a fair question, and I'd answer it this way: we remain focused on execution.",
    "I'd caution against reading too much into a single quarter.",
];

/// Answers to questions about a particular Comparable.
pub const PEER_EVASIONS: [&str; 3] = [
    "We don't comment on individual Comparables.",
    "We wish them well. Their results are their own.",
    "I'd note that one quarter does not make a trend.",
];

pub const OPERATOR_CLOSING: [&str; 2] = [
    "This concludes today's conference call. Thank you for participating. You may now disconnect.",
    "Ladies and gentlemen, that concludes today's call. You may now disconnect.",
];

pub const CLOSING_REMARK: [&str; 2] = [
    "Thank you all for your questions. We look forward to speaking with you next quarter.",
    "Thank you, everyone. We remain confident in our direction.",
];
