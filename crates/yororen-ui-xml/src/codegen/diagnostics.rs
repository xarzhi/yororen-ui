pub(crate) fn edit_distance(a: &str, b: &str) -> usize {
    let a = a.chars().collect::<Vec<_>>();
    let b = b.chars().collect::<Vec<_>>();
    let n = a.len();
    let m = b.len();
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr = vec![0usize; m + 1];
    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                curr[j] = curr[j].min(prev[j - 2] + cost);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}
pub(crate) fn did_you_mean<'a>(name: &str, candidates: &[&'a str]) -> Option<&'a str> {
    let name = name.to_ascii_lowercase();
    let mut best: Option<(&'a str, usize)> = None;
    for c in candidates {
        // Exact prefix match beats edit distance.
        let lower = c.to_ascii_lowercase();
        let dist = if lower.starts_with(&name) || name.starts_with(&lower) {
            1
        } else {
            edit_distance(&name, &lower)
        };
        let threshold = (name.len() / 2).max(2);
        if dist <= threshold && best.is_none_or(|(_, d)| dist < d) {
            best = Some((*c, dist));
        }
    }
    best.map(|(c, _)| c)
}
