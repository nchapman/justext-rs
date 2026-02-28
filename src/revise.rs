// Port of revise_paragraph_classification() from Python jusText justext/core.py

use std::collections::HashMap;

use crate::paragraph::{ClassType, Paragraph};

/// Context-sensitive revision of paragraph classifications.
///
/// Assumes `classify_paragraphs` has already set `initial_class` on all paragraphs.
/// Runs four stages in order; each stage mutates `class_type`.
pub fn revise_paragraph_classification(paragraphs: &mut [Paragraph], max_heading_distance: usize) {
    // Stage 1: copy initial_class → class_type, then promote short headings near good blocks.
    for i in 0..paragraphs.len() {
        paragraphs[i].class_type = paragraphs[i].initial_class;

        if !(paragraphs[i].heading && paragraphs[i].class_type == ClassType::Short) {
            continue;
        }

        let mut j = i + 1;
        let mut distance = 0;
        while j < paragraphs.len() && distance <= max_heading_distance {
            if paragraphs[j].class_type == ClassType::Good {
                paragraphs[i].class_type = ClassType::NearGood;
                break;
            }
            distance += paragraphs[j].text.chars().count();
            j += 1;
        }
    }

    // Stage 2: classify Short paragraphs by neighbors (BATCHED — changes applied after loop).
    let mut new_classes: HashMap<usize, ClassType> = HashMap::new();
    for i in 0..paragraphs.len() {
        if paragraphs[i].class_type != ClassType::Short {
            continue;
        }
        let prev = get_neighbour(i, paragraphs, true, Direction::Prev);
        let next = get_neighbour(i, paragraphs, true, Direction::Next);

        let class = if prev == ClassType::Good && next == ClassType::Good {
            ClassType::Good
        } else if prev == ClassType::Bad && next == ClassType::Bad {
            ClassType::Bad
        } else if (prev == ClassType::Bad
            && get_neighbour(i, paragraphs, false, Direction::Prev) == ClassType::NearGood)
            || (next == ClassType::Bad
                && get_neighbour(i, paragraphs, false, Direction::Next) == ClassType::NearGood)
        {
            ClassType::Good
        } else {
            ClassType::Bad
        };
        new_classes.insert(i, class);
    }
    for (i, c) in new_classes {
        paragraphs[i].class_type = c;
    }

    // Stage 3: classify NearGood paragraphs (NOT batched — changes apply immediately).
    for i in 0..paragraphs.len() {
        if paragraphs[i].class_type != ClassType::NearGood {
            continue;
        }
        let prev = get_neighbour(i, paragraphs, true, Direction::Prev);
        let next = get_neighbour(i, paragraphs, true, Direction::Next);
        paragraphs[i].class_type = if prev == ClassType::Bad && next == ClassType::Bad {
            ClassType::Bad
        } else {
            ClassType::Good
        };
    }

    // Stage 4: promote non-bad headings near good blocks to Good.
    for i in 0..paragraphs.len() {
        if !(paragraphs[i].heading
            && paragraphs[i].class_type == ClassType::Bad
            && paragraphs[i].initial_class != ClassType::Bad)
        {
            continue;
        }

        let mut j = i + 1;
        let mut distance = 0;
        while j < paragraphs.len() && distance <= max_heading_distance {
            if paragraphs[j].class_type == ClassType::Good {
                paragraphs[i].class_type = ClassType::Good;
                break;
            }
            distance += paragraphs[j].text.chars().count();
            j += 1;
        }
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Prev,
    Next,
}

/// Walk in the given direction, skipping Short paragraphs always,
/// and NearGood paragraphs when `ignore_neargood` is true.
/// Returns Bad if no qualifying neighbor is found (document edge default).
fn get_neighbour(
    i: usize,
    paragraphs: &[Paragraph],
    ignore_neargood: bool,
    direction: Direction,
) -> ClassType {
    let len = paragraphs.len();
    let mut idx = i as isize;
    loop {
        idx = match direction {
            Direction::Prev => idx - 1,
            Direction::Next => idx + 1,
        };
        if idx < 0 || idx >= len as isize {
            return ClassType::Bad;
        }
        let c = paragraphs[idx as usize].class_type;
        match c {
            ClassType::Good | ClassType::Bad => return c,
            ClassType::NearGood if !ignore_neargood => return c,
            ClassType::Short | ClassType::NearGood => continue, // skip
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paragraph::ClassType::*;

    /// Build a minimal Paragraph with the given initial_class (and class_type = initial_class).
    fn para(cf: ClassType) -> Paragraph {
        let mut p = Paragraph::new(
            "body.p".to_string(),
            "/html[1]/body[1]/p[1]".to_string(),
            "some text here".to_string(),
            0,
            0,
        );
        p.initial_class = cf;
        p.class_type = cf;
        p
    }

    fn para_heading(cf: ClassType) -> Paragraph {
        let mut p = Paragraph::new(
            "body.h1".to_string(),
            "/html[1]/body[1]/h1[1]".to_string(),
            "heading text".to_string(),
            0,
            0,
        );
        p.initial_class = cf;
        p.class_type = cf;
        p.heading = true;
        p
    }

    fn para_text(cf: ClassType, text: &str) -> Paragraph {
        let mut p = Paragraph::new(
            "body.p".to_string(),
            "/html[1]/body[1]/p[1]".to_string(),
            text.to_string(),
            0,
            0,
        );
        p.initial_class = cf;
        p.class_type = cf;
        p
    }

    // --- Stage 1 ---

    #[test]
    fn test_stage1_short_heading_near_good_becomes_neargood_then_good() {
        // Stage 1 promotes short heading → NearGood.
        // Stage 3 then sees NearGood with neighbor Good → Good.
        // Final result is Good — correct Python-faithful behavior.
        let mut ps = vec![para_heading(Short), para(Good)];
        revise_paragraph_classification(&mut ps, 200);
        assert_eq!(ps[0].class_type, Good);
    }

    #[test]
    fn test_stage1_short_heading_not_promoted_becomes_bad() {
        // Good is beyond max_heading_distance → heading stays Short after stage 1.
        // Stage 2 then classifies Short: prev=Bad(edge), next=Bad(long para) → Bad.
        let mut ps = vec![
            para_heading(Short),
            para_text(Bad, &"x".repeat(201)),
            para(Good),
        ];
        revise_paragraph_classification(&mut ps, 200);
        assert_eq!(ps[0].class_type, Bad);
    }

    #[test]
    fn test_stage1_non_heading_short_not_promoted() {
        let mut ps = vec![para(Short), para(Good)];
        revise_paragraph_classification(&mut ps, 200);
        // Short non-heading: stage 1 doesn't touch it; stage 2 classifies by neighbors
        // Neighbors: prev=Bad (edge), next=Good → mixed → check neargood proximity → Bad
        assert_eq!(ps[0].class_type, Bad);
    }

    // --- Stage 2 ---

    #[test]
    fn test_stage2_short_between_two_good() {
        let mut ps = vec![para(Good), para(Short), para(Good)];
        revise_paragraph_classification(&mut ps, 200);
        assert_eq!(ps[1].class_type, Good);
    }

    #[test]
    fn test_stage2_short_between_two_bad() {
        let mut ps = vec![para(Bad), para(Short), para(Bad)];
        revise_paragraph_classification(&mut ps, 200);
        assert_eq!(ps[1].class_type, Bad);
    }

    #[test]
    fn test_stage2_short_neargood_proximity_prev() {
        // Mixed case: prev(ignore=true)=Good, next(ignore=true)=Bad.
        // next is Bad; check next(ignore=false): next is NearGood → Good.
        // [Good, Short, NearGood, Bad]
        let mut ps = vec![para(Good), para(Short), para(NearGood), para(Bad)];
        revise_paragraph_classification(&mut ps, 200);
        assert_eq!(ps[1].class_type, Good);
    }

    #[test]
    fn test_stage2_short_neargood_proximity_next() {
        // Mixed case: prev(ignore=true)=Bad, next(ignore=true)=Good.
        // prev is Bad; check prev(ignore=false): prev is NearGood → Good.
        // [Bad, NearGood, Short, Good]
        let mut ps = vec![para(Bad), para(NearGood), para(Short), para(Good)];
        revise_paragraph_classification(&mut ps, 200);
        assert_eq!(ps[2].class_type, Good);
    }

    #[test]
    fn test_stage2_batching() {
        // Two adjacent short paragraphs — changes must not cascade within the pass.
        // [Good, Short, Short, Bad]
        // Both shorts: prev(Good/Good), next(Bad/Bad) → mixed for both individually
        // Short[1]: prev=Good, next=Bad → mixed → check neargood → no neargood → Bad
        // Short[2]: prev=Good (skips Short[1] since batch hasn't applied), next=Bad → Bad
        let mut ps = vec![para(Good), para(Short), para(Short), para(Bad)];
        revise_paragraph_classification(&mut ps, 200);
        // Both resolve as Bad (no neargood neighbors)
        assert_eq!(ps[1].class_type, Bad);
        assert_eq!(ps[2].class_type, Bad);
    }

    // --- Stage 3 ---

    #[test]
    fn test_stage3_neargood_both_bad_neighbors() {
        let mut ps = vec![para(Bad), para(NearGood), para(Bad)];
        revise_paragraph_classification(&mut ps, 200);
        assert_eq!(ps[1].class_type, Bad);
    }

    #[test]
    fn test_stage3_neargood_one_good_neighbor() {
        let mut ps = vec![para(Good), para(NearGood), para(Bad)];
        revise_paragraph_classification(&mut ps, 200);
        assert_eq!(ps[1].class_type, Good);
    }

    #[test]
    fn test_stage3_neargood_at_document_end() {
        // NearGood at end: next neighbor = Bad (edge default)
        let mut ps = vec![para(Good), para(NearGood)];
        revise_paragraph_classification(&mut ps, 200);
        assert_eq!(ps[1].class_type, Good); // prev=Good, next=Bad(edge) → not both bad → Good
    }

    // --- Stage 4 ---

    #[test]
    fn test_stage4_heading_bad_cf_not_bad_near_good() {
        // Heading with initial_class=Short, was revised to Bad, near Good → promoted to Good
        let mut ps = vec![
            para_heading(Short),
            para_text(Bad, &"x".repeat(10)),
            para(Good),
        ];
        // Manually set up: heading cf=Short, class=Bad (simulating stage 2 made it Bad)
        ps[0].class_type = Bad;
        revise_paragraph_classification(&mut ps, 200);
        // Stage 1 runs first: short heading near good → neargood, not Bad going into stage 4
        // Actually let's use cf=NearGood so stage 1 doesn't touch it
        // Rebuild: heading with cf=NearGood, class=Bad
        let mut ps2 = vec![
            {
                let mut p = para_heading(NearGood);
                p.class_type = Bad;
                p
            },
            para_text(Bad, "filler"),
            para(Good),
        ];
        revise_paragraph_classification(&mut ps2, 200);
        assert_eq!(ps2[0].class_type, Good);
    }

    #[test]
    fn test_stage4_heading_cf_bad_not_promoted() {
        // Heading with initial_class=Bad stays Bad even near Good (initial_class=Bad is excluded)
        let mut ps = vec![para_heading(Bad), para(Good)];
        revise_paragraph_classification(&mut ps, 200);
        assert_eq!(ps[0].class_type, Bad);
    }

    // --- Neighbor helper edge cases ---

    #[test]
    fn test_neighbour_at_start_returns_bad() {
        let ps = vec![para(Short), para(Good)];
        let prev = get_neighbour(0, &ps, true, Direction::Prev);
        assert_eq!(prev, Bad);
    }

    #[test]
    fn test_neighbour_at_end_returns_bad() {
        let ps = vec![para(Good), para(Short)];
        let next = get_neighbour(1, &ps, true, Direction::Next);
        assert_eq!(next, Bad);
    }
}
