use std::{fs::read_to_string, time::Instant};

use charming::{
    component::{Axis, Grid, Legend},
    element::{
        AxisPointer, AxisPointerType, AxisType, Emphasis, EmphasisFocus, LineStyle, LineStyleType,
        MarkLine, MarkLineData, MarkLineVariant, Tooltip, Trigger,
    },
    series::{bar, Bar, Sankey, Series},
    Chart, HtmlRenderer,
};

use book_helper::Book;
use charming::{
    element::ItemStyle,
    series::{Pie, PieRoseType},
};
use file_helper::read_file;
use folder::{merge_process, single_process};
use itertools::Itertools;
use s3_helper::Bucket;

use crate::registry::Registry;

mod book_helper;
pub mod color;
pub mod discontinuity_detector;
mod file_helper;
mod folder;
mod hit_counter;
pub mod item;
pub mod line;
mod ortho;
mod registry;
mod s3_helper;

#[tokio::main]
pub async fn add(file_name: String, endpoint: String, location: String) {
    let text = read_to_string(&file_name).unwrap();
    let bucket = Bucket::new(endpoint, location).await;

    if bucket.bucket_does_not_exist().await {
        bucket.create_bucket().await;
    }

    let body = read_file(&file_name);
    bucket.save_to_bucket_top_level(&file_name, body).await;

    let chunks: Vec<_> = text.split("CHAPTER").collect();
    let mut chunk_number = 0;
    for chunk_text in chunks {
        chunk_number += 1;
        let book_chunk = Book::book_from_text(&file_name, chunk_text, chunk_number);
        let registry = Registry::from_book(&book_chunk);
        bucket.write_chunk(registry).await;
    }
    bucket.delete_from_bucket_top_level(&file_name).await;
}

#[tokio::main]
pub async fn get(endpoint: String, location: String) {
    let bucket = Bucket::new(endpoint, location).await;

    bucket.dump_results().await;
}

#[tokio::main]
pub async fn delete(endpoint: String, location: String) {
    let bucket = Bucket::new(endpoint, location).await;

    bucket.delete_largest_answer().await
}

#[tokio::main]
pub async fn process(endpoint: String, location: String) {
    let bucket = Bucket::new(endpoint, location).await;
    let mut all_answers = vec![]; // this should not be saved in its entirety long term. Ideally a report would be saved
    let mut answer_connections: Vec<(String, String, String)> = vec![];
    loop {
        dbg!();
        if let Some(registry) = bucket.checkout_smallest_chunk().await {
            let ans: Registry = single_process(&registry);
            all_answers.push(ans.clone());

            bucket.save_answer(ans).await;
            bucket.delete_chunk(registry).await;
            display_report(&all_answers, &answer_connections);
        } else if let Some((source_answer, target_answer)) =
            bucket.checkout_largest_and_smallest_answer().await
        {
            let new_answer = merge_process(&source_answer, &target_answer);
            all_answers.push(new_answer.clone());
            answer_connections.push((
                source_answer.name.clone(),
                target_answer.name.clone(),
                new_answer.name.clone(),
            ));
            display_report(&all_answers, &answer_connections);
            bucket.save_answer(new_answer).await;
            bucket.delete_answer(source_answer).await;
            bucket.delete_answer(target_answer).await;
        } else {
            break;
        }
    }
}

fn display_report(all_answers: &Vec<Registry>, answer_connections: &Vec<(String, String, String)>) {
    dbg!("updating report");
    let all_names = all_answers.iter().map(|r| r.name.clone()).collect_vec();
    let stream_names = all_answers
        .iter()
        .flat_map(|r| {
            let name = r.name.clone();
            r.count_by_shape()
                .iter()
                .map(|(shape, _)| {
                    let mut to_add = name.clone();
                    to_add.push_str(&shape.iter().map(|x| x.to_string()).join(","));
                    to_add
                })
                .collect_vec()
        })
        .chain(all_names)
        .collect_vec();

    let single_links = all_answers
        .iter()
        .flat_map(|r| {
            r.clone()
                .count_by_shape()
                .clone()
                .into_iter()
                .map(|(shape, count)| {
                    let mut to_add = r.name.clone();
                    to_add.push_str(&shape.iter().map(|x| x.to_string()).join(","));
                    let link = (r.name.clone(), to_add.clone(), count as i32);
                    link
                })
                .collect_vec()
        })
        .collect_vec();

    // todo come back for multiple links (combine)
    dbg!(&stream_names);
    dbg!(&single_links);
    let chart = Chart::new().series(
        Sankey::new()
            .emphasis(Emphasis::new().focus(EmphasisFocus::Adjacency))
            .data(stream_names)
            .links(single_links),
    );
    let mut renderer = charming::ImageRenderer::new(1000, 800);
    renderer.save(&chart, "sankey.svg").unwrap();

    // let mut renderer = HtmlRenderer::new("my charts", 1000, 800);
    // renderer.save(&chart, "chart.html").unwrap();
}
