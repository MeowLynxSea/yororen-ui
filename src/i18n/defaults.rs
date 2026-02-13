//! Default placeholder translations helper.
//!
//! This module provides functions to get default placeholders for components.

use crate::i18n::Locale;

/// Default placeholders for UI components.
pub struct DefaultPlaceholders;

impl DefaultPlaceholders {
    /// Get the default placeholder for a Select component.
    pub fn select_placeholder(locale: &Locale) -> &'static str {
        match locale.language() {
            "zh" => "请选择…",
            "ja" => "選択…",
            "ko" => "선택…",
            "ar" => "اختر…",
            "he" => "בחר…",
            "fr" => "Sélectionner…",
            "de" => "Auswählen…",
            "es" => "Seleccionar…",
            _ => "Select…",
        }
    }

    /// Get the default placeholder for a ComboBox search.
    pub fn combobox_search_placeholder(locale: &Locale) -> &'static str {
        match locale.language() {
            "zh" => "搜索…",
            "ja" => "検索…",
            "ko" => "검색…",
            "ar" => "بحث…",
            "he" => "חפש…",
            "fr" => "Rechercher…",
            "de" => "Suchen…",
            "es" => "Buscar…",
            _ => "Search…",
        }
    }

    /// Get the default label for a DropdownMenu.
    pub fn dropdown_menu_label(locale: &Locale) -> &'static str {
        match locale.language() {
            "zh" => "菜单",
            "ja" => "メニュー",
            "ko" => "메뉴",
            "ar" => "قائمة",
            "he" => "תפריט",
            "fr" => "Menu",
            "de" => "Menü",
            "es" => "Menú",
            _ => "Menu",
        }
    }

    /// Get the default placeholder for a FilePathInput.
    pub fn file_path_placeholder(locale: &Locale) -> &'static str {
        match locale.language() {
            "zh" => "请选择路径…",
            "ja" => "パスを選択…",
            "ko" => "경로 선택…",
            "ar" => "اختر مسار…",
            "he" => "בחר נתיב…",
            "fr" => "Sélectionner un chemin…",
            "de" => "Pfad auswählen…",
            "es" => "Seleccionar ruta…",
            _ => "Select a path…",
        }
    }

    /// Get the default placeholder for a KeybindingInput.
    pub fn keybinding_press_keys(locale: &Locale) -> &'static str {
        match locale.language() {
            "zh" => "请按下按键…",
            "ja" => "キーを押してください…",
            "ko" => "키를 누르세요…",
            "ar" => "اضغط المفاتيح…",
            "he" => "לחץ על מקשים…",
            "fr" => "Appuyez sur les touches…",
            "de" => "Tasten drücken…",
            "es" => "Presiona las teclas…",
            _ => "Press keys…",
        }
    }

    /// Get the waiting text for a KeybindingInput.
    pub fn keybinding_waiting(locale: &Locale) -> &'static str {
        match locale.language() {
            "zh" => "等待输入…",
            "ja" => "入力を待っています…",
            "ko" => "입력 대기 중…",
            "ar" => "في انتظار الإدخال…",
            "he" => "ממתין לקלט…",
            "fr" => "En attente d'entrée…",
            "de" => "Warte auf Eingabe…",
            "es" => "Esperando entrada…",
            _ => "Waiting for keys…",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_placeholder() {
        let en = Locale::new("en").unwrap();
        let zh = Locale::new("zh-CN").unwrap();

        assert_eq!(DefaultPlaceholders::select_placeholder(&en), "Select…");
        assert_eq!(DefaultPlaceholders::select_placeholder(&zh), "请选择…");
    }
}
