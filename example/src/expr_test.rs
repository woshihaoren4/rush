#[cfg(test)]
mod test {
    use rush_core::{RuleFlow, Rush};
    use rush_expr_engine::ExprEngine;
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use std::collections::HashMap;

    const SIMPLE_RULE: &'static str = "
    rule COMPLEX_RULE
    when
        age > 18
    then
        stage = '成人'
    ";

    #[test]
    fn test_simple_rule() {
        let rh = Rush::from(Into::<ExprEngine>::into([SIMPLE_RULE]));

        let res: HashMap<String, String> =
            rh.flow(r#"{"age":19}"#.parse::<Value>().unwrap()).unwrap();
        assert_eq!(res.get("stage").unwrap().as_str(), "成人");

        let res: HashMap<String, String> =
            rh.flow(r#"{"age":18}"#.parse::<Value>().unwrap()).unwrap();
        assert_eq!(res.get("stage"), None);
    }

    const ORDER: &'static str = r#"{"order_id":"78135346576251344","shop_channel":"eleme","status":2,"logistic_type":"堂食","store_id":"78135346875149325","mobile":"187****1234","remark":"不放糖","payment":{"total_amount":1000,"real_pay_amount":800,"discount_amount":200},"products":[{"name":"酱香拿铁","spu_id":"78135346576987856","sku_id":"78135346587513154","quantity":2,"price":250},{"name":"黑凤梨","spu_id":"78135346658764184","sku_id":"78135346784318321","quantity":1,"price":500}],"coupons":[{"name":"会员专属八折券","code":"DE34-DFAS-13KD-XX34","quantity":1,"price":200}]}"#;

    #[derive(Serialize, Deserialize)]
    struct Product {
        name: String,
        quantity: i64,
        price: i64,
    }
    #[derive(Serialize, Deserialize)]
    struct Coupon {
        name: String,
        quantity: i64,
        price: i64,
    }

    fn product_total_amount(ps: Vec<Product>) -> anyhow::Result<i64> {
        let mut total_amount = 0;
        for i in ps.iter() {
            total_amount += i.quantity * i.price
        }
        Ok(total_amount)
    }
    fn coupon_discount_amount(cs: Vec<Coupon>) -> anyhow::Result<i64> {
        let mut total_amount = 0;
        for i in cs.iter() {
            total_amount += i.quantity * i.price
        }
        Ok(total_amount)
    }

    const COMPLEX_RULE: &'static str = "
    rule COMPLEX_RULE
    when
        shop_channel == 'eleme';
        status == 2; /*支付完成*/
        logistic_type == '堂食';
        payment.total_amount == payment.real_pay_amount + payment.discount_amount;
        payment.real_pay_amount ==  payment.total_amount - payment.discount_amount;
        - payment.total_amount ==  1000 - 2000;
        payment.discount_amount * 2 == 400;
        payment.discount_amount / 2 == 100;
        payment.total_amount == product_total_amount(products);
        payment.discount_amount == coupon_discount_amount(coupons);
        2>>1 == 1;
        1<<2 == 4;
        1 | 2 == 3;
        3 & 2 == 2;
        contain([1,2,3,4],status) && !contain([5],status);
        sub([1,2,3,4],[1]) && !sub([1,2,3,4],[5]);
        1 ^ 2 == 3;
        ~ 1 == -2;
        true;
        true || false;
    then
        message = 'success'
    ";
    #[test]
    fn test_complex_rule() {
        let rh = Rush::from(Into::<ExprEngine>::into([COMPLEX_RULE]))
            .register_function("product_total_amount", product_total_amount)
            .register_function("coupon_discount_amount", coupon_discount_amount);

        let res: HashMap<String, String> = rh.flow(ORDER.parse::<Value>().unwrap()).unwrap();
        assert_eq!(res.get("message").unwrap().as_str(), "success");
    }
}
