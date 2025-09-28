use esa_rust::acc::dynamic_accumulator::DynamicAccumulator;

fn main() {
    println!("=== 集合并集与交集证明演示 ===\n");

    // 1. 定义两个原始集合的明文值
    let set1_values = vec![100, 200, 300];
    let set2_values = vec![200, 300, 400];
    println!("集合 1: {:?}", set1_values);
    println!("集合 2: {:?}\n", set2_values);

    // 2. 创建第一个累加器
    let mut acc1 = DynamicAccumulator::new();
    acc1.add_batch(&set1_values).unwrap();
    println!("累加器 1 值: {:?}", acc1.acc_value);
    
    // 3. 创建第二个累加器
    let mut acc2 = DynamicAccumulator::new();
    acc2.add_batch(&set2_values).unwrap();
    println!("累加器 2 值: {:?}\n", acc2.acc_value);
    
    // 4. 作为证明者 (Prover)，计算并集和交集，并生成证明
    println!("正在生成并集证明...");
    let (union_values, intersection_values, union_acc, union_proof) = acc1
        .prove_union_with_values(&acc2, &set1_values, &set2_values)
        .unwrap();
    
    println!("证明生成完毕！");
    println!(" -> 计算出的并集: {:?}", union_values);
    println!(" -> 计算出的交集: {:?}", intersection_values);
    println!(" -> 并集累加器值: {:?}\n", union_acc.acc_value);

    // 5. 作为验证者 (Verifier)，验证整个证明
    println!("正在验证并集证明...");
    let is_valid = DynamicAccumulator::verify_union_with_values(
        acc1.acc_value,
        acc2.acc_value,
        &union_values,
        &intersection_values,
        &union_proof,
    );

    if is_valid {
        println!("并集证明验证成功！");
        println!("证明确认：");
        println!("  - 提供的交集 {:?} 是正确的。", intersection_values);
        println!("  - 提供的并集 {:?} 是正确的。", union_values);
    } else {
        println!("并集证明验证失败！");
    }
}
