use serde::{Deserialize, Serialize};

use crate::country::Country;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub tasks: Vec<TaskConfig>,
}

#[derive(Debug, Deserialize)]
pub struct TaskConfig {
    pub product: String,
    pub profiles: Vec<Profile>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub email: String,
    pub phone: String,
    pub card: Card,
    pub delivery: Address,
    pub billing: Address,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub number: String,
    pub expiry_month: i64,
    pub expiry_year: i64,
    pub cvv: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub first_name: String,
    pub last_name: String,
    pub address1: String,
    pub address2: Option<String>,
    pub zip: String,
    pub city: String,
    pub country: Country,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSAddress<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub country: &'static FPSCountry,
    pub address_line1: &'a str,
    pub address_line2: &'a str,
    pub address_line3: &'a str,
    pub city: FPSCity<'a>,
    pub state: FPSState<'a>,
    pub zip_code: &'a str,
    pub phone: &'a str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSCity<'a> {
    pub name: &'a str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSState<'a> {
    pub name: &'a str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSCountry {
    pub id: isize,
    pub name: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSPatchAddress<'a> {
    pub billing_address: FPSAddress<'a>,
    pub shipping_address: FPSAddress<'a>,
}

///////////////////////

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSProduct {
    pub image_groups: Vec<ImageGroup>,
    pub price: Option<Price>,
    pub result: ProductResult,
    pub recommended_set: i64,
    pub slug: String,
    pub scale_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageGroup {
    pub order: i64,
    pub images: Vec<Image>,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub size: String,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub price_excl_taxes: f64,
    pub price_incl_taxes: f64,
    pub price_incl_taxes_without_discount: f64,
    pub discount_excl_taxes: f64,
    pub discount_incl_taxes: f64,
    pub discount_rate: f64,
    pub taxes_rate: f64,
    pub taxes_value: f64,
    pub tags: Vec<String>,
    pub formatted_price: String,
    pub formatted_price_without_discount: String,
    pub formatted_price_without_currency: String,
    pub formatted_price_without_discount_and_currency: String,
    pub tax_type: String,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductResult {
    pub id: i64,
    pub short_description: String,
    pub tag: i64,
    pub variants: Vec<Variant>,
    pub has_parent_product: bool,
    pub parent_product_id: i64,
    pub made_in: String,
    pub is_online: bool,
    pub is_exclusive: bool,
    pub is_customizable: bool,
    pub style_id: i64,
    pub scale_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variant {
    pub id: String,
    pub merchant_id: i64,
    pub formatted_price: String,
    // pub formatted_price_without_discount: String,
    // pub purchase_channel: i64,
    pub quantity: i64,
    pub size: String,
    // pub scale: String,
    // pub scale_abbreviation: String,
    // pub size_description: String,
    // pub is_one_size: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSOrder {
    pub id: i64,
    pub checkout_order: FPSCheckoutOrder,
    // pub shipping_options: Vec<ShippingOption>,
    pub payment_methods: FPSPaymentMethods,
    pub order_status: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSCheckoutOrder {
    pub country_id: i64,
    pub created_date: String,
    pub currency: String,
    pub customer_type: i64,
    pub grand_total: f64,
    pub id: i64,
    pub locale: String,
    pub order_id: String,
    pub status: i64,
    pub sub_total_amount: f64,
    pub sub_total_amount_excl_taxes: f64,
    pub total_discount: f64,
    pub total_quantity: i64,
    pub total_shipping_fee: f64,
    pub total_taxes: f64,
    pub total_domestic_taxes: f64,
    pub total_credit: f64,
    pub formatted_grand_total: String,
    pub formatted_sub_total_amount: String,
    pub formatted_sub_total_amount_excl_taxes: String,
    pub formatted_total_discount: String,
    pub formatted_total_shipping_fee: String,
    pub formatted_total_taxes: String,
    pub formatted_total_domestic_taxes: String,
    pub formatted_total_credit: String,
    pub payment_intent_id: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSPaymentMethods {
    pub customer_accounts: Vec<FPSCustomerAccount>,
    pub credit_card: FPSCreditCardList,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSCustomerAccount {
    #[serde(rename = "type")]
    pub type_field: String,
    pub id: String,
    pub description: String,
    pub code: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSCreditCardList {
    #[serde(rename = "type")]
    pub type_field: String,
    pub credit_cards: Vec<FPSCreditCard>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSCreditCard {
    pub id: String,
    pub description: String,
    pub code: String,
}

//////
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSCreateOrder<'a> {
    pub guest_user_email: &'a str,
    pub use_payment_intent: bool,
    pub shipping_mode: &'a str,
    pub items: Vec<FPSItem>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSItem {
    pub merchant_id: i64,
    pub product_id: i64,
    pub quantity: i64,
    pub variant_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSCardPaymentIntent<'a> {
    pub card_number: &'a str,
    pub card_holder_name: &'a str,
    pub card_expiry_month: i64,
    pub card_expiry_year: i64,
    pub card_cvv: &'a str,
    pub payment_method_type: &'a str,
    pub payment_method_id: &'a str,
    pub save_payment_method_as_token: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FPSCreatedIntent {
    pub created_at: String,
}
