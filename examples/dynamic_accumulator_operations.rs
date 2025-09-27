use esa_rust::acc::dynamic_accumulator::{DynamicAccumulator, QueryResult};

fn main() {
    // 1. 创建一个新的动态累加器
    let mut acc = DynamicAccumulator::new();
    println!("初始化空的累加器完成。");
    println!("初始累加器值: {:?}\n", acc.acc_value);

    // 2. 添加元素并验证证明
    println!("向累加器中添加 100...");
    let add_proof_100 = acc.add(&100).unwrap();
    println!("添加完成，正在验证操作证明...");
    assert!(add_proof_100.verify());
    println!("证明有效！");
    println!("当前累加器值: {:?}\n", acc.acc_value);

    println!("向累加器中添加 200...");
    let add_proof_200 = acc.add(&200).unwrap();
    println!("添加完成，正在验证操作证明...");
    assert!(add_proof_200.verify());
    println!("证明有效！");
    println!("当前累加器值: {:?}\n", acc.acc_value);
    println!("--------------------------------------------------\n");

    // 3. 查询元素 200 (应存在)
    println!("查询元素 200...");
    match acc.query(&200) {
        QueryResult::Membership(proof) => {
            println!("找到元素 200，正在验证成员资格证明...");
            assert!(acc.verify_membership(&proof));
            println!("证明有效！\n");
        }
        _ => panic!("查询 200 失败，本应找到成员资格证明"),
    }

    // 4. 查询元素 999 (应不存在)
    println!("查询元素 999...");
    match acc.query(&999) {
        QueryResult::NonMembership(proof) => {
            println!("元素 999 不存在，正在验证非成员资格证明...");
            assert!(acc.verify_non_membership(&proof));
            println!("证明有效！\n");
        }
        _ => panic!("查询 999 失败，本应找到非成员资格证明"),
    }
    println!("--------------------------------------------------\n");

    // 5. 更新元素 100 -> 150，并验证证明
    println!("更新元素 100 -> 150...");
    let (delete_proof, add_proof) = acc.update(&100, &150).unwrap();
    println!("更新完成，正在验证删除旧元素的操作证明...");
    assert!(delete_proof.verify());
    println!("删除证明有效！");

    println!("正在验证添加新元素的操作证明...");
    assert!(add_proof.verify());
    println!("添加证明有效！");
    println!("当前累加器值: {:?}\n", acc.acc_value);
    println!("--------------------------------------------------\n");

    // 6. 验证 100 已被删除，150 已被添加
    println!("验证更新结果...");
    assert!(acc.prove_membership(&100).is_err(), "100 本应被删除");
    println!("确认：元素 100 已不存在。");
    assert!(acc.prove_membership(&150).is_ok(), "150 本应存在");
    println!("确认：元素 150 已成功添加。");
    assert!(acc.prove_membership(&200).is_ok(), "200 应该仍然存在");
    println!("确认：元素 200 未受影响。\n");
    println!("--------------------------------------------------\n");

    // 7. 删除元素 150，并验证证明
    println!("删除元素 150...");
    let delete_proof_150 = acc.delete(&150).unwrap();
    println!("删除完成，正在验证操作证明...");
    assert!(delete_proof_150.verify());
    println!("证明有效！");
    println!("当前累加器值: {:?}\n", acc.acc_value);
    println!("--------------------------------------------------\n");

    // 8. 验证 150 已被删除
    println!("验证删除结果...");
    assert!(acc.prove_membership(&150).is_err(), "150 本应被删除");
    println!("确认：元素 150 已不存在。\n");

    println!("所有操作和证明均已成功验证！");
}
