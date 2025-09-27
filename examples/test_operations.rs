use vchain::acc::dynamic_accumulator::DynamicAccumulator;

fn main() {
    // 初始化一个空的动态累加器
    let mut dyn_acc = DynamicAccumulator::new();
    println!("成功创建了一个空的动态累加器。");
    println!("初始累加器值: {:?}", dyn_acc.acc_value);
    println!("--------------------------------------------------");

    // 1. 添加元素
    println!("向累加器中添加元素 100, 200, 100...");
    dyn_acc.add(&100);
    dyn_acc.add(&200);
    dyn_acc.add(&100); // 重复添加 100
    println!("添加操作完成。");
    println!("当前累加器值: {:?}", dyn_acc.acc_value);
    println!("--------------------------------------------------");

    // 2. 证明成员资格
    println!("为元素 200 生成成员资格证明...");
    match dyn_acc.prove_membership(&200) {
        Ok(proof) => {
            println!("成功生成成员资格证明。");
            let is_valid = dyn_acc.verify_membership(&proof);
            println!("验证证明是否有效: {}", is_valid);
            assert!(is_valid);
        }
        Err(e) => println!("生成证明失败: {}", e),
    }
    println!("--------------------------------------------------");

    // 3. 证明非成员资格
    println!("为元素 300 生成非成员资格证明...");
    match dyn_acc.prove_non_membership(&300) {
        Ok(proof) => {
            println!("成功生成非成员资格证明。");
            let is_valid = dyn_acc.verify_non_membership(&proof);
            println!("验证证明是否有效: {}", is_valid);
            assert!(is_valid);
        }
        Err(e) => println!("生成证明失败: {}", e),
    }
    println!("--------------------------------------------------");

    // 4. 尝试为存在的元素生成非成员资格证明（预期会失败）
    println!("尝试为元素 100 生成非成员资格证明（预期会失败）...");
    match dyn_acc.prove_non_membership(&100) {
        Ok(_) => println!("意外地成功生成了证明。"),
        Err(e) => println!("按预期生成证明失败: {}", e),
    }
    println!("--------------------------------------------------");

    // 5. 删除元素
    println!("从累加器中删除一个元素 100...");
    if let Err(e) = dyn_acc.delete(&100) {
        println!("删除失败: {}", e);
    } else {
        println!("删除成功。");
        println!("当前累加器值: {:?}", dyn_acc.acc_value);
    }
    println!("--------------------------------------------------");

    // 6. 再次为 100 生成成员资格证明（仍然存在一个 100）
    println!("再次为元素 100 生成成员资格证明...");
    match dyn_acc.prove_membership(&100) {
        Ok(proof) => {
            println!("成功生成成员资格证明。");
            let is_valid = dyn_acc.verify_membership(&proof);
            println!("验证证明是否有效: {}", is_valid);
            assert!(is_valid);
        }
        Err(e) => println!("生成证明失败: {}", e),
    }
    println!("--------------------------------------------------");

    // 7. 删除最后一个 100
    println!("从累加器中删除最后一个元素 100...");
    if let Err(e) = dyn_acc.delete(&100) {
        println!("删除失败: {}", e);
    } else {
        println!("删除成功。");
        println!("当前累加器值: {:?}", dyn_acc.acc_value);
    }
    println!("--------------------------------------------------");

    // 8. 此时为 100 生成成员资格证明（预期会失败）
    println!("在完全删除后，为元素 100 生成成员资格证明（预期会失败）...");
    match dyn_acc.prove_membership(&100) {
        Ok(_) => println!("意外地成功生成了证明。"),
        Err(e) => println!("按预期生成证明失败: {}", e),
    }
    println!("--------------------------------------------------");

    println!("所有操作测试完毕。");
}
