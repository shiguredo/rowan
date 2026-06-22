//! ベンチマーク専用の最小 `Language` 実装

use shiguredo_rowan::{Language, SyntaxKind};

/// ベンチマーク専用の最小 Language
//
// 複数のベンチマークターゲットで共有されるモジュールのため、ターゲットによっては未使用になる。
// #[expect] はターゲットごとに lint の発火有無が異なるため使えず、 #[allow] を使用する。
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BenchLang;

/// ベンチマーク専用の SyntaxKind 列挙子
///
/// `NODE`, `TOKEN`, `TARGET` の raw 値は `benches/common/input.rs` の定数と対応する。
//
// BenchLang と同じ理由で #[allow(dead_code)] を使用する。
#[allow(dead_code)]
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BenchKind {
    /// 内部ノード
    Node = 0,
    /// 葉トークン
    Token = 1,
    /// api ベンチで `first_child_by_kind` の最悪ケースを計測するための kind
    Target = 2,
}

impl Language for BenchLang {
    type Kind = BenchKind;

    fn kind_from_raw(raw: SyntaxKind) -> Self::Kind {
        match raw.0 {
            0 => BenchKind::Node,
            1 => BenchKind::Token,
            2 => BenchKind::Target,
            other => panic!("invalid raw SyntaxKind {other} for BenchLang (this is a bug)"),
        }
    }

    fn kind_to_raw(kind: Self::Kind) -> SyntaxKind {
        SyntaxKind(kind as u16)
    }
}
