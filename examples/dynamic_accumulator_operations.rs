use vchain::acc::dynamic_accumulator::DynamicAccumulator;

fn main() {
    // 1. 创建一个新的动态累加器
    let mut acc = DynamicAccumulator::new();
    println!("初始化空的累加器完成。");
    println!("初始累加器值: {:?}", acc.acc_value);
    println!("--------------------------------------------------\n");

    // 2. 添加元素
    println!("向累加器中添加 100 和 200...");
    acc.add(&100);
    acc.add(&200);
    println!("添加完成。");
    println!("当前累加器值: {:?}", acc.acc_value);
    println!("--------------------------------------------------\n");

    // 3. 为 200 生成成员资格证明
    println!("为 200 生成成员资格证明...");
    let proof = acc.prove_membership(&200).unwrap();
    println!("证明生成成功: ");
    println!("  - 元素: {:?}", proof.element);
    println!("  - 见证 (Witness): {:?}", proof.witness);
    println!("\n");

    // 4. 验证该证明
    let is_valid = acc.verify_membership(&proof);
    println!("验证 200 的成员资格证明...");
    println!("证明是否有效? {}", is_valid);
    assert!(is_valid);
    println!("--------------------------------------------------\n");

    // 5. 删除 200
    println!("从累加器中删除 200...");
    acc.delete(&200).unwrap();
    println!("删除完成。");
    println!("当前累加器值: {:?}", acc.acc_value);
    println!("\n");

    // 6. 再次使用旧证明进行验证 (应该会失败)
    let is_valid_after_delete = acc.verify_membership(&proof);
    println!("在删除 200 后，再次使用旧证明进行验证...");
    println!("证明是否有效? {}", is_valid_after_delete);
    assert!(!is_valid_after_delete);
    println!("--------------------------------------------------\n");

    // 7. 尝试为已删除的元素生成证明 (应该会失败)
    println!("尝试为已删除的 200 生成新证明...");
    let proof_result = acc.prove_membership(&200);
    match &proof_result {
        Ok(_) => println!("错误：不应能为已删除的元素生成证明。"),
        Err(e) => println!("成功捕获错误: {}", e),
    }
    assert!(proof_result.is_err());
    println!("--------------------------------------------------\n");

    println!("示例程序执行完毕。");
}
