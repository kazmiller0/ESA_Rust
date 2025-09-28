use esa_rust::acc::dynamic_accumulator::DynamicAccumulator;

fn main() {
    println!("=== 集合交集证明演示 ===\n");

    // 1. 创建第一个累加器，包含元素 {100, 200, 300}
    let mut acc1 = DynamicAccumulator::new();
    println!("创建累加器 1，添加元素 100, 200, 300...");
    acc1.add(&100).unwrap();
    acc1.add(&200).unwrap();
    acc1.add(&300).unwrap();
    println!("累加器 1 值: {:?}", acc1.acc_value);
    println!("累加器 1 包含 {} 个元素\n", acc1.len());

    // 2. 创建第二个累加器，包含元素 {200, 300, 400}
    let mut acc2 = DynamicAccumulator::new();
    println!("创建累加器 2，添加元素 200, 300, 400...");
    acc2.add(&200).unwrap();
    acc2.add(&300).unwrap();
    acc2.add(&400).unwrap();
    println!("累加器 2 值: {:?}", acc2.acc_value);
    println!("累加器 2 包含 {} 个元素\n", acc2.len());

    // 3. 计算交集并生成证明
    println!("计算累加器 1 和累加器 2 的交集...");
    let (intersection_acc, intersection_proof) = acc1.prove_intersection(&acc2).unwrap();
    println!("交集计算完成！");
    println!("交集累加器值: {:?}", intersection_acc.acc_value);
    println!("交集包含 {} 个元素", intersection_acc.len());
    println!("交集证明已生成\n");

    // 4. 验证交集证明
    println!("验证交集证明...");
    let is_valid = DynamicAccumulator::verify_intersection(
        acc1.acc_value,
        acc2.acc_value,
        intersection_acc.acc_value,
        &intersection_proof,
    );
    
    if is_valid {
        println!("交集证明验证成功！");
        println!("证明确认：交集累加器确实代表了两个原始累加器的交集。\n");
    } else {
        println!("交集证明验证失败！");
        return;
    }

    // 5. 验证交集中的具体元素
    println!("验证交集中的具体元素...");
    
    // 验证 200 在交集中
    match intersection_acc.query(&200) {
        esa_rust::acc::dynamic_accumulator::QueryResult::Membership(proof) => {
            if intersection_acc.verify_membership(&proof) {
                println!("元素 200 在交集中，成员资格证明有效");
            }
        }
        _ => println!("元素 200 不在交集中"),
    }

    // 验证 300 在交集中
    match intersection_acc.query(&300) {
        esa_rust::acc::dynamic_accumulator::QueryResult::Membership(proof) => {
            if intersection_acc.verify_membership(&proof) {
                println!("元素 300 在交集中，成员资格证明有效");
            }
        }
        _ => println!("元素 300 不在交集中"),
    }

    // 验证 100 不在交集中
    match intersection_acc.query(&100) {
        esa_rust::acc::dynamic_accumulator::QueryResult::NonMembership(proof) => {
            if intersection_acc.verify_non_membership(&proof) {
                println!("元素 100 不在交集中，非成员资格证明有效");
            }
        }
        _ => println!("元素 100 意外地在交集中"),
    }

    // 验证 400 不在交集中
    match intersection_acc.query(&400) {
        esa_rust::acc::dynamic_accumulator::QueryResult::NonMembership(proof) => {
            if intersection_acc.verify_non_membership(&proof) {
                println!("元素 400 不在交集中，非成员资格证明有效");
            }
        }
        _ => println!("元素 400 意外地在交集中"),
    }

    println!("\n=== 演示完成 ===");
    println!("总结：");
    println!("- 累加器 1: {{100, 200, 300}}");
    println!("- 累加器 2: {{200, 300, 400}}");
    println!("- 交集: {{200, 300}}");
    println!("- 交集证明验证成功，所有成员资格和非成员资格证明都有效！");

    println!("\n=== 空交集演示 ===");
    
    // 6. 演示空交集的情况
    let mut acc3 = DynamicAccumulator::new();
    acc3.add(&500).unwrap();
    acc3.add(&600).unwrap();
    
    let mut acc4 = DynamicAccumulator::new();
    acc4.add(&700).unwrap();
    acc4.add(&800).unwrap();
    
    println!("累加器 3: {{500, 600}}");
    println!("累加器 4: {{700, 800}}");
    
    let (empty_intersection, empty_proof) = acc3.prove_intersection(&acc4).unwrap();
    println!("空交集累加器包含 {} 个元素", empty_intersection.len());
    
    let empty_is_valid = DynamicAccumulator::verify_intersection(
        acc3.acc_value,
        acc4.acc_value,
        empty_intersection.acc_value,
        &empty_proof,
    );
    
    if empty_is_valid {
        println!("空交集证明验证成功！");
    } else {
        println!("空交集证明验证失败！");
    }
}
