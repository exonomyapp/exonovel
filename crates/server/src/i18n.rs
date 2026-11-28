use axum::{
    extract::Path,
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocaleDescriptor {
    pub code: &'static str,
    pub native_name: &'static str,
    pub english_name: &'static str,
    pub dir: &'static str,
    pub region: &'static str,
    pub pack_url: String,
    pub willow_path: String,
}

struct LocaleSeed {
    code: &'static str,
    native_name: &'static str,
    english_name: &'static str,
    dir: &'static str,
    region: &'static str,
    nav_desk: &'static str,
    nav_dear_world: &'static str,
    nav_docs: &'static str,
    nav_profile: &'static str,
    nav_preferences: &'static str,
    nav_sign_out: &'static str,
    ui_lang_label: &'static str,
    input_lang_label: &'static str,
    live_sync_label: &'static str,
    profile_title: &'static str,
    preferences_title: &'static str,
    notifications_title: &'static str,
    save_label: &'static str,
}

const LOCALES: &[LocaleSeed] = &[
    LocaleSeed {
        code: "en",
        native_name: "English",
        english_name: "English",
        dir: "ltr",
        region: "Global",
        nav_desk: "Desk",
        nav_dear_world: "Dear World",
        nav_docs: "Docs & ADRs",
        nav_profile: "Profile",
        nav_preferences: "Preferences",
        nav_sign_out: "Sign out",
        ui_lang_label: "App Language",
        input_lang_label: "Writing Input",
        live_sync_label: "Live",
        profile_title: "Profile",
        preferences_title: "Preferences",
        notifications_title: "Notifications",
        save_label: "Save changes",
    },
    LocaleSeed {
        code: "de",
        native_name: "Deutsch",
        english_name: "German",
        dir: "ltr",
        region: "Europe",
        nav_desk: "Schreibtisch",
        nav_dear_world: "Dear World",
        nav_docs: "Doku & ADRs",
        nav_profile: "Profil",
        nav_preferences: "Präferenzen",
        nav_sign_out: "Abmelden",
        ui_lang_label: "App-Sprache",
        input_lang_label: "Eingabesprache",
        live_sync_label: "Echtzeit",
        profile_title: "Profil",
        preferences_title: "Präferenzen",
        notifications_title: "Benachrichtigungen",
        save_label: "Änderungen speichern",
    },
    LocaleSeed {
        code: "pl",
        native_name: "Polski",
        english_name: "Polish",
        dir: "ltr",
        region: "Europe",
        nav_desk: "Biurko",
        nav_dear_world: "Dear World",
        nav_docs: "Dokumentacja i ADR",
        nav_profile: "Profil",
        nav_preferences: "Preferencje",
        nav_sign_out: "Wyloguj się",
        ui_lang_label: "Język aplikacji",
        input_lang_label: "Język pisania",
        live_sync_label: "Na żywo",
        profile_title: "Profil",
        preferences_title: "Preferencje",
        notifications_title: "Powiadomienia",
        save_label: "Zapisz zmiany",
    },
    LocaleSeed {
        code: "fr",
        native_name: "Français",
        english_name: "French",
        dir: "ltr",
        region: "Europe",
        nav_desk: "Bureau",
        nav_dear_world: "Dear World",
        nav_docs: "Docs et ADR",
        nav_profile: "Profil",
        nav_preferences: "Préférences",
        nav_sign_out: "Se déconnecter",
        ui_lang_label: "Langue de l'interface",
        input_lang_label: "Langue de saisie",
        live_sync_label: "En direct",
        profile_title: "Profil",
        preferences_title: "Préférences",
        notifications_title: "Notifications",
        save_label: "Enregistrer",
    },
    LocaleSeed {
        code: "es",
        native_name: "Español",
        english_name: "Spanish",
        dir: "ltr",
        region: "Europe & Americas",
        nav_desk: "Escritorio",
        nav_dear_world: "Dear World",
        nav_docs: "Docs y ADRs",
        nav_profile: "Perfil",
        nav_preferences: "Preferencias",
        nav_sign_out: "Cerrar sesión",
        ui_lang_label: "Idioma de la app",
        input_lang_label: "Idioma de escritura",
        live_sync_label: "En vivo",
        profile_title: "Perfil",
        preferences_title: "Preferencias",
        notifications_title: "Notificaciones",
        save_label: "Guardar cambios",
    },
    LocaleSeed {
        code: "it",
        native_name: "Italiano",
        english_name: "Italian",
        dir: "ltr",
        region: "Europe",
        nav_desk: "Scrivania",
        nav_dear_world: "Dear World",
        nav_docs: "Docs e ADR",
        nav_profile: "Profilo",
        nav_preferences: "Preferenze",
        nav_sign_out: "Esci",
        ui_lang_label: "Lingua app",
        input_lang_label: "Lingua di scrittura",
        live_sync_label: "Dal vivo",
        profile_title: "Profilo",
        preferences_title: "Preferenze",
        notifications_title: "Notifiche",
        save_label: "Salva modifiche",
    },
    LocaleSeed {
        code: "nl",
        native_name: "Nederlands",
        english_name: "Dutch",
        dir: "ltr",
        region: "Europe",
        nav_desk: "Bureau",
        nav_dear_world: "Dear World",
        nav_docs: "Docs & ADR's",
        nav_profile: "Profiel",
        nav_preferences: "Voorkeuren",
        nav_sign_out: "Uitloggen",
        ui_lang_label: "App-taal",
        input_lang_label: "Invoertaal",
        live_sync_label: "Live",
        profile_title: "Profiel",
        preferences_title: "Voorkeuren",
        notifications_title: "Meldingen",
        save_label: "Wijzigingen opslaan",
    },
    LocaleSeed {
        code: "pt",
        native_name: "Português",
        english_name: "Portuguese",
        dir: "ltr",
        region: "Europe & Americas",
        nav_desk: "Mesa",
        nav_dear_world: "Dear World",
        nav_docs: "Docs e ADRs",
        nav_profile: "Perfil",
        nav_preferences: "Preferências",
        nav_sign_out: "Sair",
        ui_lang_label: "Idioma do app",
        input_lang_label: "Idioma de escrita",
        live_sync_label: "Ao vivo",
        profile_title: "Perfil",
        preferences_title: "Preferências",
        notifications_title: "Notificações",
        save_label: "Salvar alterações",
    },
    LocaleSeed {
        code: "hu",
        native_name: "Magyar",
        english_name: "Hungarian",
        dir: "ltr",
        region: "Europe",
        nav_desk: "Íróasztal",
        nav_dear_world: "Dear World",
        nav_docs: "Doksi és ADR-ek",
        nav_profile: "Profil",
        nav_preferences: "Preferenciák",
        nav_sign_out: "Kijelentkezés",
        ui_lang_label: "Alkalmazás nyelve",
        input_lang_label: "Beviteli nyelv",
        live_sync_label: "Élő",
        profile_title: "Profil",
        preferences_title: "Preferenciák",
        notifications_title: "Értesítések",
        save_label: "Módosítások mentése",
    },
    LocaleSeed {
        code: "ro",
        native_name: "Română",
        english_name: "Romanian",
        dir: "ltr",
        region: "Europe",
        nav_desk: "Birou",
        nav_dear_world: "Dear World",
        nav_docs: "Documente și ADR",
        nav_profile: "Profil",
        nav_preferences: "Preferințe",
        nav_sign_out: "Deconectare",
        ui_lang_label: "Limba aplicației",
        input_lang_label: "Limba de scriere",
        live_sync_label: "În direct",
        profile_title: "Profil",
        preferences_title: "Preferințe",
        notifications_title: "Notificări",
        save_label: "Salvează modificările",
    },
    LocaleSeed {
        code: "uk",
        native_name: "Українська",
        english_name: "Ukrainian",
        dir: "ltr",
        region: "Europe",
        nav_desk: "Стіл",
        nav_dear_world: "Dear World",
        nav_docs: "Документація і ADR",
        nav_profile: "Профіль",
        nav_preferences: "Вподобання",
        nav_sign_out: "Вийти",
        ui_lang_label: "Мова інтерфейсу",
        input_lang_label: "Мова введення",
        live_sync_label: "Наживо",
        profile_title: "Профіль",
        preferences_title: "Вподобання",
        notifications_title: "Сповіщення",
        save_label: "Зберегти зміни",
    },
    LocaleSeed {
        code: "ru",
        native_name: "Русский",
        english_name: "Russian",
        dir: "ltr",
        region: "Europe & Eurasia",
        nav_desk: "Стол",
        nav_dear_world: "Dear World",
        nav_docs: "Документация и ADR",
        nav_profile: "Профиль",
        nav_preferences: "Предпочтения",
        nav_sign_out: "Выйти",
        ui_lang_label: "Язык интерфейса",
        input_lang_label: "Язык ввода",
        live_sync_label: "В реальном времени",
        profile_title: "Профиль",
        preferences_title: "Предпочтения",
        notifications_title: "Уведомления",
        save_label: "Сохранить изменения",
    },
    LocaleSeed {
        code: "sr",
        native_name: "Српски",
        english_name: "Serbian",
        dir: "ltr",
        region: "Europe",
        nav_desk: "Радни сто",
        nav_dear_world: "Dear World",
        nav_docs: "Документација и ADR",
        nav_profile: "Профил",
        nav_preferences: "Преференције",
        nav_sign_out: "Одјави се",
        ui_lang_label: "Језик апликације",
        input_lang_label: "Језик писања",
        live_sync_label: "Уживо",
        profile_title: "Профил",
        preferences_title: "Преференције",
        notifications_title: "Обавештења",
        save_label: "Сачувај измене",
    },
    LocaleSeed {
        code: "el",
        native_name: "Ελληνικά",
        english_name: "Greek",
        dir: "ltr",
        region: "Europe",
        nav_desk: "Γραφείο",
        nav_dear_world: "Dear World",
        nav_docs: "Έγγραφα & ADR",
        nav_profile: "Προφίλ",
        nav_preferences: "Προτιμήσεις",
        nav_sign_out: "Αποσύνδεση",
        ui_lang_label: "Γλώσσα εφαρμογής",
        input_lang_label: "Γλώσσα γραφής",
        live_sync_label: "Ζωντανά",
        profile_title: "Προφίλ",
        preferences_title: "Προτιμήσεις",
        notifications_title: "Ειδοποιήσεις",
        save_label: "Αποθήκευση αλλαγών",
    },
    LocaleSeed {
        code: "ar",
        native_name: "العربية",
        english_name: "Arabic",
        dir: "rtl",
        region: "MENA",
        nav_desk: "المكتب",
        nav_dear_world: "Dear World",
        nav_docs: "الوثائق و ADRs",
        nav_profile: "الملف الشخصي",
        nav_preferences: "التفضيلات",
        nav_sign_out: "تسجيل الخروج",
        ui_lang_label: "لغة التطبيق",
        input_lang_label: "لغة الكتابة",
        live_sync_label: "مباشر",
        profile_title: "الملف الشخصي",
        preferences_title: "التفضيلات",
        notifications_title: "الإشعارات",
        save_label: "حفظ التغييرات",
    },
    LocaleSeed {
        code: "he",
        native_name: "עברית",
        english_name: "Hebrew",
        dir: "rtl",
        region: "MENA",
        nav_desk: "שולחן כתיבה",
        nav_dear_world: "Dear World",
        nav_docs: "מסמכים ו-ADR",
        nav_profile: "פרופיל",
        nav_preferences: "העדפות",
        nav_sign_out: "התנתקות",
        ui_lang_label: "שפת הממשק",
        input_lang_label: "שפת כתיבה",
        live_sync_label: "חי",
        profile_title: "פרופיל",
        preferences_title: "העדפות",
        notifications_title: "התראות",
        save_label: "שמור שינויים",
    },
    LocaleSeed {
        code: "zh-Hans",
        native_name: "简体中文",
        english_name: "Chinese (Simplified)",
        dir: "ltr",
        region: "East Asia",
        nav_desk: "书桌",
        nav_dear_world: "Dear World",
        nav_docs: "文档与架构决策",
        nav_profile: "个人资料",
        nav_preferences: "偏好设置",
        nav_sign_out: "退出登录",
        ui_lang_label: "界面语言",
        input_lang_label: "输入语言",
        live_sync_label: "实时",
        profile_title: "个人资料",
        preferences_title: "偏好设置",
        notifications_title: "通知",
        save_label: "保存更改",
    },
    LocaleSeed {
        code: "zh-Hant",
        native_name: "繁體中文",
        english_name: "Chinese (Traditional)",
        dir: "ltr",
        region: "East Asia",
        nav_desk: "書桌",
        nav_dear_world: "Dear World",
        nav_docs: "文件與架構決策",
        nav_profile: "個人檔案",
        nav_preferences: "偏好設定",
        nav_sign_out: "登出",
        ui_lang_label: "介面語言",
        input_lang_label: "輸入語言",
        live_sync_label: "即時",
        profile_title: "個人檔案",
        preferences_title: "偏好設定",
        notifications_title: "通知",
        save_label: "儲存變更",
    },
    LocaleSeed {
        code: "hi",
        native_name: "हिन्दी",
        english_name: "Hindi",
        dir: "ltr",
        region: "South Asia",
        nav_desk: "डेस्क",
        nav_dear_world: "Dear World",
        nav_docs: "दस्तावेज़ और ADR",
        nav_profile: "प्रोफ़ाइल",
        nav_preferences: "प्राथमिकताएँ",
        nav_sign_out: "साइन आउट",
        ui_lang_label: "ऐप भाषा",
        input_lang_label: "लेखन भाषा",
        live_sync_label: "लाइव",
        profile_title: "प्रोफ़ाइल",
        preferences_title: "प्राथमिकताएँ",
        notifications_title: "सूचनाएँ",
        save_label: "बदलाव सहेजें",
    },
    LocaleSeed {
        code: "bn",
        native_name: "বাংলা",
        english_name: "Bengali",
        dir: "ltr",
        region: "South Asia",
        nav_desk: "ডেস্ক",
        nav_dear_world: "Dear World",
        nav_docs: "ডক্স ও ADR",
        nav_profile: "প্রোফাইল",
        nav_preferences: "পছন্দসমূহ",
        nav_sign_out: "সাইন আউট",
        ui_lang_label: "অ্যাপের ভাষা",
        input_lang_label: "লেখার ভাষা",
        live_sync_label: "লাইভ",
        profile_title: "প্রোফাইল",
        preferences_title: "পছন্দসমূহ",
        notifications_title: "বিজ্ঞপ্তি",
        save_label: "পরিবর্তন সংরক্ষণ",
    },
    LocaleSeed {
        code: "mr",
        native_name: "मराठी",
        english_name: "Marathi",
        dir: "ltr",
        region: "South Asia",
        nav_desk: "डेस्क",
        nav_dear_world: "Dear World",
        nav_docs: "दस्तऐवज आणि ADR",
        nav_profile: "प्रोफाइल",
        nav_preferences: "प्राधान्ये",
        nav_sign_out: "साइन आउट",
        ui_lang_label: "अॅप भाषा",
        input_lang_label: "लेखन भाषा",
        live_sync_label: "लाइव्ह",
        profile_title: "प्रोफाइल",
        preferences_title: "प्राधान्ये",
        notifications_title: "सूचना",
        save_label: "बदल जतन करा",
    },
    LocaleSeed {
        code: "te",
        native_name: "తెలుగు",
        english_name: "Telugu",
        dir: "ltr",
        region: "South Asia",
        nav_desk: "డెస్క్",
        nav_dear_world: "Dear World",
        nav_docs: "డాక్స్ & ADRలు",
        nav_profile: "ప్రొఫైల్",
        nav_preferences: "ప్రాధాన్యతలు",
        nav_sign_out: "సైన్ అవుట్",
        ui_lang_label: "యాప్ భాష",
        input_lang_label: "రచన భాష",
        live_sync_label: "లైవ్",
        profile_title: "ప్రొఫైల్",
        preferences_title: "ప్రాధాన్యతలు",
        notifications_title: "నోటిఫికేషన్లు",
        save_label: "మార్పులను సేవ్ చేయండి",
    },
    LocaleSeed {
        code: "ta",
        native_name: "தமிழ்",
        english_name: "Tamil",
        dir: "ltr",
        region: "South Asia",
        nav_desk: "மேசை",
        nav_dear_world: "Dear World",
        nav_docs: "ஆவணங்கள் & ADR",
        nav_profile: "சுயவிவரம்",
        nav_preferences: "விருப்பத்தேர்வுகள்",
        nav_sign_out: "வெளியேறு",
        ui_lang_label: "பயன்பாட்டு மொழி",
        input_lang_label: "எழுத்து மொழி",
        live_sync_label: "நேரலை",
        profile_title: "சுயவிவரம்",
        preferences_title: "விருப்பத்தேர்வுகள்",
        notifications_title: "அறிவிப்புகள்",
        save_label: "மாற்றங்களைச் சேமி",
    },
    LocaleSeed {
        code: "sw",
        native_name: "Kiswahili",
        english_name: "Swahili",
        dir: "ltr",
        region: "Africa",
        nav_desk: "Dawati",
        nav_dear_world: "Dear World",
        nav_docs: "Nyaraka na ADR",
        nav_profile: "Wasifu",
        nav_preferences: "Mapendeleo",
        nav_sign_out: "Toka",
        ui_lang_label: "Lugha ya Programu",
        input_lang_label: "Lugha ya Kuandika",
        live_sync_label: "Moja kwa moja",
        profile_title: "Wasifu",
        preferences_title: "Mapendeleo",
        notifications_title: "Arifa",
        save_label: "Hifadhi mabadiliko",
    },
    LocaleSeed {
        code: "ha",
        native_name: "Hausa",
        english_name: "Hausa",
        dir: "ltr",
        region: "Africa",
        nav_desk: "Tebur",
        nav_dear_world: "Dear World",
        nav_docs: "Takardu da ADR",
        nav_profile: "Bayanan Kaina",
        nav_preferences: "Abubuwan da aka fi so",
        nav_sign_out: "Fita",
        ui_lang_label: "Harshen Manhaja",
        input_lang_label: "Harshen Rubutu",
        live_sync_label: "Kai tsaye",
        profile_title: "Bayanan Kaina",
        preferences_title: "Abubuwan da aka fi so",
        notifications_title: "Sanarwa",
        save_label: "Ajiye sauye-sauye",
    },
    LocaleSeed {
        code: "yo",
        native_name: "Yorùbá",
        english_name: "Yoruba",
        dir: "ltr",
        region: "Africa",
        nav_desk: "Tábìlì",
        nav_dear_world: "Dear World",
        nav_docs: "Àwọn Ìwé àti ADR",
        nav_profile: "Àpèjúwe Mi",
        nav_preferences: "Àwọn ohun ààyò",
        nav_sign_out: "Jáde",
        ui_lang_label: "Èdè Àpù",
        input_lang_label: "Èdè Ìkọ̀wé",
        live_sync_label: "Lẹ́sẹ̀kẹsẹ̀",
        profile_title: "Àpèjúwe Mi",
        preferences_title: "Àwọn ohun ààyò",
        notifications_title: "Àwọn Ìfitónilétí",
        save_label: "Fipamọ́",
    },
    LocaleSeed {
        code: "ig",
        native_name: "Igbo",
        english_name: "Igbo",
        dir: "ltr",
        region: "Africa",
        nav_desk: "Teebụl",
        nav_dear_world: "Dear World",
        nav_docs: "Akwụkwọ na ADR",
        nav_profile: "Profaịlụ",
        nav_preferences: "Mmasị",
        nav_sign_out: "Pụọ",
        ui_lang_label: "Asụsụ Ngwa",
        input_lang_label: "Asụsụ Ndetu",
        live_sync_label: "Ndụ",
        profile_title: "Profaịlụ",
        preferences_title: "Mmasị",
        notifications_title: "Ọkwa",
        save_label: "Chekwaa mgbanwe",
    },
    LocaleSeed {
        code: "am",
        native_name: "አማርኛ",
        english_name: "Amharic",
        dir: "ltr",
        region: "Africa",
        nav_desk: "ጠረጴዛ",
        nav_dear_world: "Dear World",
        nav_docs: "ሰነዶች እና ADR",
        nav_profile: "መገለጫ",
        nav_preferences: "ምርጫዎች",
        nav_sign_out: "ውጣ",
        ui_lang_label: "የመተግበሪያ ቋንቋ",
        input_lang_label: "የጽሑፍ ቋንቋ",
        live_sync_label: "ቀጥታ",
        profile_title: "መገለጫ",
        preferences_title: "ምርጫዎች",
        notifications_title: "ማሳወቂያዎች",
        save_label: "ለውጦችን አስቀምጥ",
    },
    LocaleSeed {
        code: "zu",
        native_name: "isiZulu",
        english_name: "Zulu",
        dir: "ltr",
        region: "Africa",
        nav_desk: "Idesika",
        nav_dear_world: "Dear World",
        nav_docs: "Amadokhumenti ne-ADR",
        nav_profile: "Iphrofayela",
        nav_preferences: "Okuncamelayo",
        nav_sign_out: "Phuma",
        ui_lang_label: "Ulimi Lohlelo",
        input_lang_label: "Ulimi Lokubhala",
        live_sync_label: "Bukhoma",
        profile_title: "Iphrofayela",
        preferences_title: "Okuncamelayo",
        notifications_title: "Izaziso",
        save_label: "Londoloza izshintsho",
    },
];

pub fn catalog_list() -> Vec<LocaleDescriptor> {
    LOCALES
        .iter()
        .map(|l| LocaleDescriptor {
            code: l.code,
            native_name: l.native_name,
            english_name: l.english_name,
            dir: l.dir,
            region: l.region,
            pack_url: format!("/api/i18n/{}.ftl", l.code),
            willow_path: format!("/public/i18n/{}.ftl", l.code),
        })
        .collect()
}

pub fn is_supported_locale(code: &str) -> bool {
    let trimmed = code.trim();
    LOCALES.iter().any(|l| l.code == trimmed)
}

pub fn direction_for_locale(code: &str) -> &'static str {
    let trimmed = code.trim();
    LOCALES
        .iter()
        .find(|l| l.code == trimmed)
        .map(|l| l.dir)
        .unwrap_or("ltr")
}

fn extended_ui_block(code: &str) -> &'static str {
    match code {
        "pl" => "\
nav-sign-in = Zaloguj się\n\
nav-toggle-menu = Przełącz menu nawigacji\n\
nav-privacy = Polityka prywatności\n\
nav-terms = Warunki korzystania z usługi\n\
live-connection-title = Globalne połączenie strumienia czasu rzeczywistego\n\
ui-lang-aria = Język interfejsu aplikacji\n\
input-lang-aria = Język wprowadzania tekstu\n\
notifications-aria = Powiadomienia\n\
switch-persona = Zmień personę\n\
create-personas-prompt = Utwórz persony w swoim profilu\n\
default-persona-label = Domyślna persona\n\
active-badge = Aktywna\n\
mark-all-read = Oznacz wszystkie jako przeczytane\n\
mark-read = Oznacz jako przeczytane\n\
no-notifications = Brak powiadomień.\n\
refresh-label = Odśwież\n\
cancel-label = Anuluj\n\
dismiss-label = Zamknij\n\
save-changes-short = Zapisz\n\
confirm-label = Potwierdź\n\
copy-text-label = Kopiuj tekst\n\
pen-name-label = Pseudonim:\n\
pending-did-label = Oczekujące did:peer\n\
workspace-desc = Twoje biurko pisarskie trzyma wszystko, nad czym pracujesz, pod ręką — niezależnie od tego, czy tworzysz pierwsze opowiadanie do szkoły, wymyślasz postacie ze znajomymi, czy piszesz wiersze w pojedynkę. Każda postać, oś czasu, scena i komentarz pozostają przypisane do Twojego pseudonimu, więc możesz swobodnie dzielić się pomysłami, uczyć się razem i mieć pewność, że Twoje teksty zawsze należą do Ciebie.\n\
btn-profile-billing = Profil, persony i rozliczenia\n\
btn-connected-apps = Połączone aplikacje i Dokumenty Google\n\
btn-open-cms = Otwórz MADR Docs CMS\n\
workspace-key-label = Aktywny klucz obszaru roboczego persony (did:peer)\n\
choose-image-btn = Wybierz zdjęcie profilowe\n\
persona-choose-avatar-btn = Wybierz zdjęcie persony\n\
desk-start-eyebrow = Rozpocznij nowy projekt\n\
desk-start-title = Co chcesz dzisiaj napisać?\n\
desk-starter-novel-hint = Gotowe na półce: Konspekt fabuły · Karty postaci · Szkic 1. rozdziału · Oś czasu świata.\n\
desk-starter-article-hint = Gotowe na półce: Teza otwierająca · Konspekt sekcji · Notatki źródłowe · Szkic eseju.\n\
desk-starter-short-hint = Gotowe na półce: Haczyk otwarcia · Punkt zwrotny · Scena postaci · Szkic opowiadania.\n\
desk-starter-script-hint = Gotowe na półce: Nagłówek sceny · Obsada i głosy · Plan scen · Strony scenariusza.\n\
desk-starter-poetry-hint = Gotowe na półce: Szkic strofy · Notatki rytmu · Kolejność w tomiku · Dedykacja autora.\n\
desk-blocks-eyebrow = Elementy składowe historii\n\
desk-blocks-title = Twoja półka z elementami opowieści\n\
desk-block-chars-title = Postacie\n\
desk-block-chars-desc = Zbieraj bohaterów, rywali i ich historie — możesz nawet zaprosić postać znajomego do wspólnej sceny z czytelnym przypisaniem autorstwa.\n\
desk-block-plots-title = Wątki i fabuła\n\
desk-block-plots-desc = Rozpisuj punkty zwrotne, rozdziały i niespodzianki, aby opowieść płynęła gładko od początku do finału.\n\
desk-block-timelines-title = Osie czasu\n\
desk-block-timelines-desc = Układaj, co wydarzyło się najpierw, co potem, a co przed laty — idealne przy kryminałach, trylogiach i projektach szkolnych.\n\
desk-block-themes-title = Motywy i tematy\n\
desk-block-themes-desc = Zapisuj główne pytania, nastrój i motywy przewodnie historii, aby cały zespół pisał w jednym tonie.\n\
desk-scratchpad-eyebrow = Brudnopis\n\
desk-scratchpad-label = Szybki notatnik pomysłów\n\
desk-scratchpad-title = Zapisz szybki pomysł\n\
desk-note-title-label = Roboczy tytuł lub temat\n\
desk-note-title-ph = np. Deszcz na schodach biblioteki...\n\
desk-note-body-label = Pierwsze zdania lub szkic sceny\n\
desk-note-body-ph = Zapisz kilka zdań, sprawdź dialog albo przełącz język pisania powyżej...\n\
desk-note-keep-btn = Przypnij pomysł na biurku\n\
desk-note-clear-btn = Wyczyść\n\
desk-note-pinned-msg = Pomysł przypięty na Twoim biurku na czas tej sesji\n\
guides-eyebrow = Dear World · Przewodniki użytkownika\n\
guides-sidebar-title = Pierwsze kroki\n\
guides-sidebar-sub = Przewodniki krok po kroku dla uczniów i studentów odkrywających wspólne pisanie w ExoNovel.\n\
guides-open-desk-btn = Przejdź do Biurka\n\
guides-signin-btn = Zaloguj się, aby zacząć pisać\n\
guides-loading = Otwieranie przewodnika Dear World...\n\
guides-empty = Wybierz rozdział przewodnika z listy, aby zacząć.\n\
toast-did-copied = Skopiowano ID did:peer\n\
toast-switched-persona = Przełączono personę\n\
oauth-connected-google = Pomyślnie połączono Dysk i Dokumenty Google.\n\
theme-system-label = Motyw systemowy\n\
theme-light-label = Jasny motyw\n\
theme-dark-label = Ciemny motyw\n\
role-admin = Administrator\n\
role-moderator = Moderator\n\
role-user = Użytkownik\n\
status-reader = Czytelnik\n\
status-pending = Oczekuje\n\
status-suspended = Zawieszone\n\
status-restricted = Ograniczone\n\
status-manager = Menedżer\n\
status-member = Członek\n\
landing-eyebrow = ExoNovel · Web, Desktop i Mobile\n\
landing-title-row1 = Wspólne pisanie kreatywne\n\
landing-title-row2 = W którym autorzy zachowują własność swoich dzieł\n\
landing-title-row3 = ... z natury!\n\
landing-lead = ExoNovel to platforma do wspólnego pisania kreatywnego i lokalno-pierwszego publikowania dla autorów, współtwórców i zespołów literackich. Twórz powieści, eseje, opowiadania, scenariusze i poezję razem, zachowując kryptograficzną własność manuskryptów.\n\
landing-cta = Zaloguj się do ExoNovel\n\
landing-cap-aria = Możliwości platformy\n\
landing-prev-slide = Poprzedni slajd\n\
landing-next-slide = Następny slajd\n\
slide-1-tab = Twoje teksty, Twoje prawa\n\
slide-1-title = Twórzcie razem, zachowując własność każdego swojego pomysłu\n\
slide-1-p1 = ExoNovel to przestrzeń do twórczego pisania, w której możesz tworzyć historie samodzielnie lub wspólnie ze znajomymi, współautorami i grupami pisarskimi w czasie rzeczywistym. Zamiast gubić informację o autorstwie w chaotycznych wspólnych dokumentach, każdy profil postaci, konspekt fabuły, oś czasu, motyw i scena pozostają wyraźnie i bezpiecznie przypisane do swojego twórcy — włącznie z komentarzami, które dodajesz przy pracach innych osób.\n\
slide-1-p2 = Współpraca nigdy więc nie oznacza utraty własnej pracy. Sam decydujesz, co pozostaje prywatne, co Twoja grupa może współedytować, a co inni pisarze mogą pożyczać i rozwijać we własnych historiach — z Twoją w pełni niezależną, zdecentralizowaną tożsamością przypisaną na stałe do tego, co stworzyłeś.\n\
slide-2-tab = Jeden twórca, wiele Person\n\
slide-2-title = Jedna tożsamość i tyle Person, ile tylko zechcesz\n\
slide-2-p1 = Pisanie powieści sci-fi ze znajomymi, publikowanie poezji pod pseudonimem i szkicowanie reportaży pod prawdziwym nazwiskiem nie powinno zmuszać Cię do żonglowania trzema osobnymi kontami. ExoNovel pozwala tworzyć wiele Person — każda ma własną nazwę wyświetlaną lub uchwyt, biogram i awatar, działa też jako proxy publikacji (możesz publikować jako wybrana Persona) oraz ma własny dostęp do społeczności czytelników i współpracowników, dzięki czemu możesz nawet rozmawiać jako Persona o swoich i cudzych tekstach. Twórcy ExoNovel nie muszą ujawniać światu swojej prawdziwej tożsamości, choć mogą to zrobić, jeśli chcą — wszystko pod jednym niezależnym logowaniem.\n\
slide-2-p2 = Przełączaj Persony jednym kliknięciem, aby od razu przejść do osobnej przestrzeni roboczej Persony z własnymi projektami, czatami grupowymi i gatunkami. Współtwórcy i czytelnicy widzą dokładnie tę nazwę Persony, którą wybrałeś dla danej przestrzeni, a wszystko, co napiszesz we wszystkich swoich Personach, pozostaje uporządkowane i trwale prawnie Twoje dzięki bezpiecznemu, zdecentralizowanemu identyfikatorowi P2P Twojej tożsamości.\n\
slide-3-tab = Pisz, gdziekolwiek jesteś\n\
slide-3-title = Online i offline — zsynchronizuj wszystkie swoje urządzenia\n\
slide-3-p1 = Twórz bezpośrednio w przeglądarce zawsze, gdy masz dostęp do sieci, lub kontynuuj pisanie bez internetu dzięki naszym darmowym aplikacjom na systemy Linux, macOS, Windows, Android i iOS. Nasze darmowe aplikacje pomagają synchronizować pracę z wersją przeglądarkową, a za niewielką opłatą miesięczną (lub jeszcze niższą w rozliczeniu rocznym) Twoje dzieła online i lokalne są zawsze pod ręką w jednym wirtualnym domu pisarskim.\n\
slide-3-p2 = Zacznij scenę w telefonie w drodze, dopracuj rozdział na laptopie później i ręcznie synchronizuj pracę na żądanie między urządzeniami offline a siecią, kiedy tylko chcesz — albo włącz automatyczną synchronizację, aby każdy szkic był aktualny na wszystkich ekranach.\n\
slide-4-tab = Łącz po swojemu\n\
slide-4-title = Podłącz swoje narzędzia i zachowaj pełną swobodę\n\
slide-4-p1 = Twoje pisanie nie powinno być uwięzione na jednej platformie. Loguj się tak, jak wolisz, pisz w formatach dopasowanych do Twojego projektu i łącz zewnętrzne dyski w chmurze, narzędzia oraz procesy robocze na potrzeby kopii zapasowych, szybkiego eksportu, publikacji, projektów wieloplatformowych lub czegokolwiek innego, czego wymaga Twój proces twórczy.\n\
slide-4-p2 = W ExoNovel wszystko współpracuje ze sobą bez zamykania Cię w jednym ekosystemie. Dodawaj połączenia zawsze, gdy pomagają, łącz narzędzia wraz z rozwojem projektów i odłączaj dowolną usługę w każdej chwili, zachowując pełny dostęp do swojego konta, Person i dzieł.\n\
landing-modes-title = W sieci, na Twoim urządzeniu i razem w synchronizacji\n\
landing-modes-sub = Pisz za darmo w przeglądarce i w naszych aplikacjach na urządzenia. Współpraca między pisarzami synchronizuje się automatycznie w sieci i na urządzeniach za darmo — a drobna opłata miesięczna lub roczna dotyczy wyłącznie automatycznej synchronizacji Twojej własnej półki między przeglądarką a urządzeniami.\n\
mode-free-badge = Zawsze za darmo\n\
mode-free-title = W przeglądarce i na urządzeniu\n\
mode-free-desc = Twórz, zapraszaj współautorów i pisz wspólnie w przeglądarce oraz w darmowych aplikacjach na Linuxa, macOS, Windows, Androida i iOS — z darmową automatyczną synchronizacją współpracy między pisarzami w sieci i na urządzeniach.\n\
mode-monthly-badge = Synchronizacja osobista\n\
mode-monthly-title = Synchronizacja osobista — $9 / mies.\n\
mode-monthly-desc = Utrzymuj własne szkice w przeglądarce i na urządzeniach offline w automatycznej synchronizacji end-to-end ($9/mies. lub równowartość w lokalnej walucie).\n\
mode-annual-badge = Roczna synchronizacja osobista\n\
mode-annual-title = Synchronizacja osobista — $90 / rok\n\
mode-annual-desc = Pełna automatyczna synchronizacja między Twoim profilem w sieci a własnymi urządzeniami ($7,50/mies. efektywnie w lokalnej walucie — dwa miesiące gratis względem planu miesięcznego).\n\
landing-formats-title = Wybierz format i zacznij tworzyć\n\
landing-formats-sub = Niezależnie od tego, czy piszesz pierwszą powieść, wypracowanie do szkoły czy scenariusz krótkiego filmu — zacznij w przejrzystym układzie stworzonym dla Twojego stylu pisania.\n\
format-novel-title = Powieści\n\
format-novel-desc = Twórz całe światy od zera. Porządkuj bohaterów, zwroty akcji i rozdziały w jednym miejscu, żeby wielkie pomysły nigdy się nie pogubiły.\n\
format-article-title = Artykuły i literatura faktu\n\
format-article-desc = Zamieniaj pomysły i materiały w mocne eseje, wpisy na bloga lub prace szkolne dzięki czytelnym konspektom i źródłom.\n\
format-short-title = Opowiadania\n\
format-short-desc = Dopracuj haczyk, zwrot akcji i finał — idealne do krótkich form, opowiadań i własnych zbiorów historii.\n\
format-screenplay-title = Scenariusze i skrypty\n\
format-screenplay-desc = Pisz sceny i dialogi wyglądające jak prawdziwe scenariusze filmowe lub wideo — formatowane automatycznie podczas pisania.\n\
format-poetry-title = Poezja\n\
format-poetry-desc = Dopasowuj rytm, strofy i podziały wersów, które zostają dokładnie tam, gdzie je ustawisz — od jednego wiersza po cały tomik.\n\
signin-back-home = ← Wstecz\n\
signin-title = Zaloguj się do ExoNovel\n\
signin-subtitle = Uwierzytelnij się, aby rozwiązać swoją autorytatywną tożsamość did:peer i otworzyć obszar roboczy.\n\
signin-checking-session = Sprawdzanie istniejącej sesji...\n\
signin-invite-badge = Osobiste zaproszenie\n\
signin-invited-group = zaprosił(a) Cię do grupy w ExoNovel:\n\
signin-invited-platform = zaprosił(a) Cię do dołączenia do ExoNovel.\n\
signin-create-heading = Utwórz konto\n\
signin-continue-heading = Kontynuuj do ExoNovel\n\
signin-or-email = lub przez e-mail\n\
signin-name-label = Imię i nazwisko\n\
signin-email-label = E-mail\n\
signin-password-label = Hasło\n\
signin-authenticating = Uwierzytelnianie...\n\
signin-create-btn = Utwórz konto\n\
signin-submit-btn = Zaloguj się\n\
signin-have-account = Masz już konto? Zaloguj się\n\
signin-need-account = Nie masz konta? Utwórz je\n\
signin-auth-failed = Uwierzytelnianie nie powiodło się.\n\
footer-brand-prefix = ExoNovel, od\n\
privacy-page-title = Polityka prywatności — ExoNovel\n\
privacy-effective = Data wejścia w życie i ostatnia aktualizacja: 24 listopada 2026\n\
privacy-heading = Polityka prywatności ExoNovel\n\
privacy-intro = ExoNovel (https://exonovel.exosystems.net) to platforma do wspólnego pisania kreatywnego i lokalno-pierwszego publikowania prowadzona przez Exosystems (https://exosystems.net). Niniejsza Polityka prywatności wyjaśnia, w jaki sposób zbieramy, wykorzystujemy, przechowujemy, chronimy i usuwamy dane osobowe oraz dane użytkownika Google.\n\
privacy-s1-title = 1. Zbierane informacje\n\
privacy-s1-intro = Zbieramy wyłącznie dane niezbędne do uwierzytelniania konta, działania obszaru roboczego i realizacji funkcji włączonych przez użytkownika:\n\
privacy-s1-google-label = Dane konta logowania Google (openid, email, profile): \n\
privacy-s1-google-body = Podczas logowania przez Google otrzymujemy zweryfikowany adres e-mail, status weryfikacji, nazwę wyświetlaną, imię i nazwisko, URL zdjęcia profilowego, język oraz identyfikator konta Google.\n\
privacy-s1-drive-label = Opcjonalne dane połączonego Dysku Google: \n\
privacy-s1-drive-body = Podstawowe logowanie nigdy nie żąda uprawnień Dysku Google. Dopiero po wyraźnym połączeniu Dysku Google w Preferencjach lub Dokumentacji ExoNovel uzyskuje dostęp do wyliczania folderów, tworzenia kopii zapasowych Markdown/ADR i przywracania lub usuwania plików wybranych w obszarze roboczym.\n\
privacy-s1-profile-label = Inne dane logowania i profilu autora: \n\
privacy-s1-profile-body = Przy logowaniu przez GitHub lub e-mail przechowujemy nazwę wyświetlaną, pseudonim, biogram, stronę WWW, lokalizację, preferowany format, preferencje języków interfejsu i pisania, strefę czasową, preferencje powiadomień, awatary (64px, 256px, 1024px) oraz klucz publiczny / did:peer.\n\
privacy-s1-works-label = Manuskrypty i zapisy współpracy: \n\
privacy-s1-works-body = Dzieła twórcze, biblie opowieści, persony, członkostwa w grupach współpracy, polecenia oraz zaproszenia utworzone lub udostępnione w ExoNovel.\n\
privacy-s1-sec-label = Bezpieczeństwo i diagnostyka techniczna: \n\
privacy-s1-sec-body = Znaczniki czasu logowania, adres IP, nagłówek User-Agent przeglądarki oraz logi błędów używane wyłącznie do ochrony kont i stabilności usługi.\n\
privacy-s2-title = 2. Pliki cookie i pamięć przeglądarki\n\
privacy-s2-body = ExoNovel używa wyłącznie niezbędnych plików cookie sesji uwierzytelniania (HttpOnly) oraz tymczasowej pamięci sesji przeglądarki do zachowania oczekujących kodów zaproszeń podczas przekierowań logowania. Nie stosujemy zewnętrznych plików cookie reklamowych ani śledzących.\n\
privacy-s3-title = 3. Sposób wykorzystania danych\n\
privacy-s3-li1 = Uwierzytelnianie sesji i rozwiązywanie tożsamości did:peer w przeglądarce, na komputerach i urządzeniach mobilnych.\n\
privacy-s3-li2 = Wyświetlanie wybranej persony, pseudonimu, awatara i preferencji językowych (w tym układu RTL).\n\
privacy-s3-li3 = Obsługa wspólnych grup literackich, atrybucji współautorów i zaproszeń inicjowanych przez użytkownika.\n\
privacy-s3-li4 = Wykonywanie inicjowanych przez użytkownika kopii zapasowych na połączonym Dysku Google.\n\
privacy-s3-li5 = Wysyłka włączonych przez użytkownika wiadomości e-mail transakcyjnych, bezpieczeństwa i powiadomień (z notes@exosystems.net, noreply@exosystems.net lub support@exosystems.net).\n\
privacy-s3-li6 = Zapobieganie nadużyciom, nieautoryzowanemu dostępowi i naruszeniom Regulaminu.\n\
privacy-s4-title = 4. Zasady danych użytkowników Google API (Limited Use) i zakaz AI NCII\n\
privacy-s4-intro = Wykorzystanie i przekazywanie informacji otrzymanych z interfejsów Google API jest w pełni zgodne z polityką Google API Services User Data Policy, w tym wymaganiami Limited Use.\n\
privacy-s4-li1-label = Brak reklam i sprzedaży danych: \n\
privacy-s4-li1-body = Nigdy nie sprzedajemy, nie wynajmujemy ani nie przekazujemy danych użytkowników Google ani Twoich tekstów brokerom danych ani platformom reklamowym.\n\
privacy-s4-li2-label = Brak trenowania ogólnych modeli AI: \n\
privacy-s4-li2-body = ExoNovel nie wykorzystuje danych użytkowników Google, plików Dysku Google ani prywatnych manuskryptów do trenowania ogólnych modeli AI lub ML.\n\
privacy-s4-li3-label = Brak generowania obrazów i zero tolerancji dla AI NCII: \n\
privacy-s4-li3-body = ExoNovel jest platformą tekstową. Nie generuje obrazów, wideo ani dźwięku i bezwzględnie zakazuje używania ExoNovel lub Google API do tworzenia lub dystrybucji AI NCII.\n\
privacy-s4-li4-label = Ograniczenia dostępu personelu: \n\
privacy-s4-li4-body = Personel Exosystems nie czyta prywatnych plików na Dysku Google ani manuskryptów, chyba że wyrazisz wyraźną zgodę na wsparcie techniczne, wymaga tego bezpieczeństwo lub prawo.\n\
privacy-s5-title = 5. Przechowywanie danych, bezpieczeństwo i ochrona kryptograficzna\n\
privacy-s5-body = Cały ruch do ExoNovel jest szyfrowany przez HTTPS/TLS. Dane webowe i stan konta przechowywane są w zarządzanych bazach danych PostgreSQL i zarządzanych usługach uwierzytelniania na infrastrukturze chmurowej klasy korporacyjnej wraz z dostarczaniem poczty transakcyjnej. Opcjonalne tokeny połączonych aplikacji zewnętrznych przechowywane po stronie serwera są szyfrowane kopertowo przez dedykowane zarządzanie kluczami i odizolowane od głównych danych logowania. W natywnych instalacjach wykorzystujących synchronizację peer-to-peer Willow i iroh dane są szyfrowane end-to-end na urządzeniu.\n\
privacy-s6-title = 6. Cofanie dostępu, eksport i usuwanie danych\n\
privacy-s6-li1-label = Odłącz i cofnij dostęp Google w dowolnym momencie: \n\
privacy-s6-li1-body = Możesz odłączyć Dysk Google w Preferencjach -> Połączone aplikacje (co natychmiast unieważnia tokeny przez https://oauth2.googleapis.com/revoke bez usuwania konta ExoNovel) lub w ustawieniach konta Google: https://myaccount.google.com/permissions.\n\
privacy-s6-li2-label = Wnioski o eksport i usunięcie konta: \n\
privacy-s6-li2-body = Możesz zażądać pełnego eksportu lub trwałego usunięcia konta, pisząc na support@exosystems.net lub exonomyapp@gmail.com. Zweryfikowane zgłoszenia realizujemy w ciągu 30 dni.\n\
privacy-s7-title = 7. Retencja danych, prywatność dzieci i aktualizacje\n\
privacy-s7-body = Przechowujemy dane tylko tak długo, jak konto jest aktywne lub wymagają tego przepisy. ExoNovel nie jest skierowany do dzieci poniżej 13 roku życia (lub 16 lat zgodnie z lokalnym prawem). O istotnych zmianach poinformujemy aktywnych użytkowników.\n\
privacy-s8-title = 8. Kontakt\n\
privacy-s8-body = W sprawach prywatności, danych Google lub usunięcia konta napisz na support@exosystems.net lub exonomyapp@gmail.com.\n\
terms-page-title = Warunki korzystania z usługi — ExoNovel\n\
terms-effective = Data wejścia w życie i ostatnia aktualizacja: 24 listopada 2026\n\
terms-heading = Warunki korzystania z usługi ExoNovel\n\
terms-intro = Korzystając z ExoNovel (https://exonovel.exosystems.net), prowadzonego przez Exosystems (https://exosystems.net), akceptujesz niniejsze Warunki korzystania z usługi oraz Politykę prywatności (/privacy).\n\
terms-s1-title = 1. Własność Twoich dzieł twórczych\n\
terms-s1-body = Zachowujesz pełną własność i prawa autorskie do wszystkich manuskryptów, opowieści, postaci, notatek, scenariuszy i wierszy publikowanych w ExoNovel. Udzielasz platformie wyłącznie ograniczonej licencji technicznej na przechowywanie, renderowanie i synchronizację zgodnie z Twoimi preferencjami.\n\
terms-s2-title = 2. Konta, klucze i dozwolone użytkowanie\n\
terms-s2-body = Odpowiadasz za bezpieczeństwo danych logowania i kluczy urządzenia did:peer. Musisz mieć ukończone 13 lat (lub wymagany wiek minimalny w Twojej jurysdykcji). Zabrania się naruszania bezpieczeństwa platformy i rozsyłania spamu.\n\
terms-s3-title = 3. Zabronione treści i zgodność z Google APIs ToS\n\
terms-s3-intro = ExoNovel jest platformą tekstową i nie oferuje narzędzi do generowania obrazów, wideo ani dźwięku. Egzekwujemy pełną zgodność z Google APIs Terms of Service i zero tolerancji dla szkodliwych treści:\n\
terms-s3-li1-label = Bezwzględny zakaz AI NCII: \n\
terms-s3-li1-body = Zabrania się używania ExoNovel lub połączonych interfejsów Google API do tworzenia, przechowywania lub rozpowszechniania niekonsensualnych treści intymnych generowanych przez AI (AI NCII) ani deepfake'ów.\n\
terms-s3-li2-label = Materiały nielegalne i naruszenia: \n\
terms-s3-li2-body = CSAM, treści bez zgody, nękanie, mowa nienawiści oraz naruszenia praw autorskich są surowo zabronione i skutkują natychmiastowym zablokowaniem konta.\n\
terms-s4-title = 4. Połączone aplikacje zewnętrzne\n\
terms-s4-body = Opcjonalne integracje, takie jak Dysk Google, uruchamiane są wyłącznie po wyraźnej zgodzie użytkownika i mogą zostać cofnięte w dowolnej chwili w Preferencjach -> Połączone aplikacje.\n\
terms-s5-title = 5. Dostępność usługi, plany synchronizacji i kontakt\n\
terms-s5-body = ExoNovel oferuje współpracę webową, lokalne przechowywanie na urządzeniu oraz opcjonalne plany synchronizacji osobistej. W sprawach prawnych i wsparcia napisz na support@exosystems.net lub exonomyapp@gmail.com.\n\
preferences-subtitle = Preferencje konta, języków, przeglądu synchronizacji i dostarczania powiadomień (zakupy, paywalle i plany znajdują się w Profilu).\n\
preferences-nav-aria = Sekcje preferencji\n\
sec-account = Konto i suwerenna tożsamość\n\
sec-connected-apps = Połączone aplikacje i Dokumenty Google\n\
sec-appearance-langs = Języki (29)\n\
sec-sync-storage = Sieć, urządzenie i synchronizacja współpracy\n\
sec-notifications = Kanały powiadomień\n\
sec-admin-allowlist = Lista dostępu i konta\n\
account-title = Konto i suwerenna tożsamość\n\
save-account-btn = Zapisz preferencje konta\n\
upload-avatar-label = Prześlij zdjęcie profilowe (WebP 64px / 256px / 1024px)\n\
root-identity-label = Główna tożsamość (did:peer)\n\
display-name-label = Nazwa wyświetlana\n\
pen-name-field-label = Domyślny pseudonim\n\
website-label = Strona WWW\n\
location-label = Lokalizacja i strefa czasowa\n\
preferred-format-label = Preferowany format pisania\n\
timezone-label = Strefa czasowa\n\
author-bio-label = Biogram autora\n\
format-opt-novel = Powieści\n\
format-opt-article = Artykuły i literatura faktu\n\
format-opt-short = Opowiadania\n\
format-opt-screenplay = Scenariusze\n\
format-opt-poetry = Poezja\n\
appearance-title = Architektura dwóch języków (29 języków)\n\
appearance-desc = Język aplikacji steruje menu, nawigacją i kierunkiem układu (automatyczne RTL dla arabskiego i hebrajskiego). Język pisania steruje edytorami niezależnie i zapisuje się bezpośrednio na koncie.\n\
offline-packs-title = Zdecentralizowane pakiety językowe offline\n\
offline-packs-desc = Aplikacje ExoNovel na urządzeniach pobierają wybrane pakiety językowe bezpośrednio z publicznej przestrzeni nazw Willow tylko do odczytu:\n\
save-theme-lang-btn = Zapisz preferencje językowe\n\
sync-title = Pisanie w sieci, na urządzeniu i synchronizacja współpracy\n\
sync-sub = Korzystaj z ExoNovel za darmo w przeglądarce i za darmo offline na urządzeniach. Synchronizacja współpracy między pisarzami jest darmowa w obu środowiskach.\n\
sync-m1-title = W przeglądarce (Darmowe)\n\
sync-m1-desc = Obszar roboczy w chmurze dostępny z dowolnej przeglądarki z natychmiastową współpracą między autorami i grupami.\n\
sync-m2-title = Na urządzeniu (Darmowe)\n\
sync-m2-desc = Przechowywanie offline-first na Windows, macOS, Linux, Android i iOS — pisz wszędzie i zachowaj pełną lokalną kontrolę nad biblioteką.\n\
sync-m3-title = Synchronizacja własnych urządzeń ($9/mies. · $90/rok)\n\
sync-m3-desc = Szyfrowana end-to-end synchronizacja utrzymująca Twoje własne szkice offline w stałej zgodności z Twoim profilem w przeglądarce.\n\
sync-collab-title = Współpraca z innymi pisarzami jest darmowa wszędzie\n\
sync-collab-desc = Za każdym razem, gdy dodajesz współpracowników, synchronizacja między nimi działa automatycznie i za darmo — niezależnie od tego, czy jedna osoba pisze w przeglądarce, a druga w aplikacji na komputerze lub telefonie. Opłata dotyczy wyłącznie synchronizacji Twojej własnej biblioteki między urządzeniem a profilem w sieci.\n\
sync-zk-title = Kryptograficzna prywatność zero-knowledge\n\
sync-zk-desc = Szkice synchronizowane z urządzeń oraz wspólne pakiety danych są szyfrowane end-to-end przez nadawcę przed przejściem przez infrastrukturę przekaźnika ExoNovel.\n\
sync-manage-in-profile = Zobacz plany synchronizacji w Profilu →\n\
notif-channels-title = Wieloplatformowe kanały dostarczania\n\
notif-email-prefix = Transakcyjna usługa e-mail (notes@exosystems.net)\n\
notif-email-active = · Dostarczanie e-mail aktywne\n\
notif-email-unset = · Poświadczenia poczty transakcyjnej nie są skonfigurowane\n\
notif-send-test = Wyślij powiadomienie testowe i e-mail\n\
notif-in-app-title = Strumień na żywo i dzwonek w aplikacji\n\
notif-in-app-desc = Natychmiastowe dostarczanie przez stały strumień czasu rzeczywistego.\n\
notif-web-push-title = Powiadomienia przeglądarki WWW\n\
notif-web-push-desc = Natywne banery systemowe przez Notification API przeglądarki.\n\
notif-native-push-title = Natywne powiadomienia urządzeń (Android UnifiedPush + FCM, iOS/macOS APNs, Windows WNS, Linux)\n\
notif-email-title = Transakcyjny e-mail (Notatki z ExoNovel)\n\
notif-digest-label = Częstotliwość podsumowań e-mail\n\
notif-digest-instant = Natychmiastowy e-mail\n\
notif-digest-daily = Dzienny raport\n\
notif-digest-off = E-mail wyłączony\n\
notif-security-title = Alerty bezpieczeństwa i logowania\n\
notif-security-desc = Powiadomienie, gdy suwerenna tożsamość uwierzytelni się na nowym urządzeniu.\n\
notif-maturity-title = Alerty wydań skanera dojrzałości\n\
notif-maturity-desc = Powiadomienie, gdy śledzone zależności opublikują nową wersję.\n\
notif-save-btn = Zapisz preferencje kanałów\n\
devices-title = Zarejestrowane punkty końcowe Push (AOTA)\n\
devices-sub = Zarejestruj dowolny natywny lub webowy transport powiadomień: UnifiedPush (Android/Linux), FCM (Android), APNs (iOS/macOS) lub WNS (Windows).\n\
device-platform-label = Platforma\n\
device-transport-label = Transport powiadomień\n\
device-label-field = Etykieta urządzenia\n\
device-token-label = Adres URL punktu końcowego lub token urządzenia\n\
device-token-ph = https://push.example.org/... lub token FCM/APNs\n\
device-register-btn = Zarejestruj punkt końcowy Push\n\
device-th-label = Etykieta\n\
device-th-platform = Platforma\n\
device-th-transport = Transport\n\
device-th-last-seen = Ostatnio widziane\n\
device-plat-android = Android\n\
device-plat-ios = iOS\n\
device-plat-linux = Linux\n\
device-plat-macos = macOS\n\
device-plat-windows = Windows\n\
device-plat-web = Przeglądarka WWW\n\
device-tr-unifiedpush = UnifiedPush (Otwarty standard)\n\
device-tr-fcm = FCM (Firebase Cloud Messaging)\n\
device-tr-apns = APNs (Apple Push Notification service)\n\
device-tr-wns = WNS (Windows Push Notification Services)\n\
device-tr-webpush = Web Push (VAPID)\n\
admin-allowlist-title = Lista dostępu do dokumentacji\n\
admin-allowlist-sub = Określa, kto może przeglądać MADR 4.0.0 CMS ExoNovel i na jakim poziomie. Kliknij dowolny did:peer poniżej, aby wypełnić pole.\n\
admin-grant-ph = did:peer:0z... lub e-mail\n\
admin-note-ph = Notatka\n\
admin-save-access-btn = Zapisz dostęp\n\
admin-th-user = Użytkownik\n\
admin-th-email = E-mail\n\
admin-th-status = Status\n\
admin-th-actions = Akcje\n\
admin-remove-btn = Usuń\n\
admin-owner-badge = Właściciel\n\
admin-accounts-title = Zarejestrowane konta i role\n\
admin-th-role = Rola\n\
admin-th-did-populate = did:peer (kliknij, aby wstawić)\n\
admin-th-provider = Dostawca\n\
admin-display-name-ph = Nazwa wyświetlana\n\
admin-pen-name-ph = Pseudonim\n\
admin-save-account-btn = Zapisz zmiany konta\n\
preferences-msg-avatar-size = Plik awatara musi mieć maksymalnie 2 MB.\n\
preferences-msg-avatar-processing = Przetwarzanie wariantów awatara...\n\
preferences-msg-avatar-saved = Awatar zapisany.\n\
preferences-msg-saved = Preferencje zapisane.\n\
preferences-toast-saved = Zapisano preferencje\n\
preferences-toast-notif-saved = Zaktualizowano kanały powiadomień\n\
preferences-toast-device-reg = Zarejestrowano urządzenie Push\n\
preferences-toast-test-sent = Wysłano powiadomienie testowe i e-mail\n\
preferences-msg-allowlist-updated = Zaktualizowano listę dostępu.\n\
preferences-toast-tier-updated = Zaktualizowano poziom dostępu\n\
preferences-toast-entry-removed = Usunięto wpis z listy dostępu\n\
preferences-msg-account-updated = Zaktualizowano konto.\n\
preferences-toast-perm = Uprawnienia powiadomień przeglądarki:\n\
connected-apps-title = Połączone aplikacje i Dokumenty Google\n\
connected-apps-desc = Podstawowe logowanie używa wyłącznie zakresu tożsamości (openid, email, profile). Połącz Dysk i Dokumenty Google tutaj, aby tworzyć, przeglądać i tworzyć kopie zapasowe manuskryptów i ADR.\n\
google-drive-docs-title = Dysk Google i Dokumenty Google\n\
connect-google-btn = Połącz Dysk i Dokumenty Google\n\
reconnect-google-btn = Połącz ponownie Google\n\
disconnect-revoke-btn = Odłącz i cofnij dostęp\n\
status-connected = Połączono\n\
status-not-connected = Niepołączono\n\
status-revoked = Cofnięto\n\
conn-primary-signin-title = Podstawowe metody logowania (tylko tożsamość)\n\
conn-loading-accounts = Ładowanie połączonych kont...\n\
conn-session-auth = Uwierzytelniono poświadczeniami sesji.\n\
conn-signin-linked = Logowanie połączone\n\
conn-scopes-prefix = Zakresy:\n\
conn-auth-acct-fallback = Autoryzowane konto Google\n\
conn-drive-optional-desc = Opcjonalna autoryzacja per plik (drive.file) do tworzenia Dokumentów Google, zapisu kopii .md i przeglądania ExoNovel Docs.\n\
conn-revoke-confirm-msg = Cofnąć tokeny OAuth ExoNovel dla Dysku i Dokumentów Google? Odłączy to kopie zapasowe Drive bez wpływu na główne logowanie do ExoNovel.\n\
conn-revoke-confirm-btn = Potwierdź cofnięcie\n\
conn-redirect-configured = Adres przekierowania OAuth (skonfigurowany u dostawcy)\n\
conn-redirect-missing = Wymagane poświadczenia Google OAuth po stronie serwera\n\
conn-copy-uri-btn = Kopiuj URI\n\
conn-toast-uri-copied = Skopiowano Redirect URI\n\
drive-ws-title = Obszar roboczy Dokumentów i Dysku Google\n\
drive-ws-sub = Twórz edytowalne Dokumenty Google z Markdown, przeglądaj foldery ExoNovel, podglądaj pliki i zarządzaj Koszem.\n\
drive-new-doc-btn = + Nowy Dokument Google\n\
drive-new-folder-btn = + Nowy folder\n\
drive-viewing-trash = Przeglądanie Kosza\n\
drive-trash-btn = Kosz\n\
drive-refresh-files = Odśwież pliki\n\
drive-ready-suffix = jest gotowy w Dokumentach Google.\n\
drive-open-gdocs = Otwórz w Dokumentach Google ↗\n\
drive-new-folder-ph = Nazwa nowego folderu...\n\
drive-create-folder-btn = Utwórz folder\n\
drive-doc-title-label = Tytuł dokumentu\n\
drive-doc-title-ph = Rozdział 1 — Konspekt\n\
drive-target-fmt-label = Format docelowy\n\
drive-fmt-gdoc = Dokument Google (natywny edytowalny)\n\
drive-fmt-md = Markdown (.md)\n\
drive-md-content-label = Treść Markdown\n\
drive-md-content-ph = # Tytuł — Wpisz scenę lub notatki w Markdown...\n\
drive-save-btn = Zapisz w Dokumentach / Dysku Google\n\
drive-my-drive-root = Mój dysk\n\
drive-search-ph = Szukaj plików na Dysku...\n\
drive-search-btn = Szukaj\n\
drive-loading-files = Ładowanie plików z Dysku Google...\n\
drive-th-name = Nazwa\n\
drive-th-type = Typ\n\
drive-th-modified = Zmodyfikowano\n\
drive-th-actions = Akcje\n\
drive-mime-folder = Folder\n\
drive-mime-gdoc = Dokument Google\n\
drive-mime-gsheet = Arkusz Google\n\
drive-mime-gslides = Prezentacje Google\n\
drive-mime-md = Markdown (.md)\n\
drive-mime-txt = Tekst\n\
drive-mime-file = Plik\n\
drive-save-inline = Zapisz\n\
drive-open-doc-btn = Otwórz Doc ↗\n\
drive-open-btn = Otwórz ↗\n\
drive-preview-btn = Podgląd\n\
drive-rename-btn = Zmień nazwę\n\
drive-restore-btn = Przywróć\n\
drive-delete-forever-btn = Usuń trwale\n\
drive-trash-item-btn = Do kosza\n\
drive-confirm-perm-delete = Potwierdzić trwałe usunięcie?\n\
drive-empty-files = Brak plików w tym widoku. Kliknij '+ Nowy Dokument Google' lub zapisz ADR z sekcji Dokumentacja.\n\
drive-close-preview = Zamknij podgląd\n\
drive-toast-doc-copied = Skopiowano treść dokumentu\n\
conn-msg-list-err = Nie udało się pobrać listy plików Dysku i Dokumentów Google.\n\
conn-msg-load-err = Nie udało się załadować połączonych aplikacji.\n\
conn-msg-revoking = Odłączanie i cofanie tokenów Google...\n\
conn-msg-revoked = Odłączono Dysk i Dokumenty Google oraz cofnięto tokeny OAuth.\n\
conn-toast-revoked = Cofnięto dostęp do Dysku i Dokumentów Google\n\
conn-msg-revoke-err = Nie udało się cofnąć autoryzacji Google.\n\
conn-msg-creating-folder = Tworzenie folderu...\n\
conn-toast-folder-created = Utworzono folder na Dysku Google\n\
conn-msg-folder-err = Tworzenie folderu nie powiodło się.\n\
conn-msg-doc-required = Podaj zarówno tytuł, jak i treść Markdown.\n\
conn-msg-converting-gdoc = Konwertowanie Markdown do natywnego Dokumentu Google...\n\
conn-msg-uploading-md = Wysyłanie pliku Markdown na Dysk Google...\n\
conn-toast-doc-saved = Zapisano dokument w Dokumentach / Dysku Google\n\
conn-msg-preview-err = Podgląd nie powiódł się.\n\
conn-toast-item-updated = Zaktualizowano element na Dysku\n\
conn-msg-action-err = Operacja nie powiodła się.\n\
tab-overview = Przegląd\n\
tab-works = Dzieła\n\
tab-groups = Grupy i zaproszenia\n\
tab-wallet = Portfel\n\
tab-paywalls = Paywalle\n\
tab-plans = Plany\n\
tab-personas = Persony\n\
tab-langs-alerts = Języki i powiadomienia\n\
profile-tabs-aria = Sekcje profilu\n\
profile-overview-title = Przegląd tożsamości twórcy\n\
profile-overview-desc = Twoja suwerenna tożsamość spaja dzieła, persony, grupy, wypłaty oraz zakupy czytelników.\n\
active-persona-label = Aktywna persona\n\
profile-sync-plan-label = Plan synchronizacji osobistej:\n\
personas-label = Persony\n\
groups-label = Grupy\n\
referrals-label = Polecenia\n\
profile-referred-by = Polecono przez:\n\
profile-personas-unit = kluczy obszaru roboczego\n\
profile-groups-unit = grup współpracy\n\
profile-referrals-unit = dołączonych pisarzy\n\
works-title = Utworzone i rozwidlone dzieła\n\
works-desc = Każdy manuskrypt przechowuje wspólne edycje Loro CRDT oraz ścieżki uprawnień Willow Meadowcap powiązane z aktywną personą.\n\
works-novels-title = Powieści i serie\n\
works-novels-desc = Rozdziały i sceny zapisywane z darmowymi lub płatnymi progami dostępu konfigurowanymi w zakładce Paywalle.\n\
works-scripts-title = Scenariusze i poezja\n\
works-scripts-desc = Wielojęzyczny edytor tekstu (29 języków + RTL) z kursorami współpracy na żywo.\n\
groups-title = Grupy i menedżerowie\n\
groups-composer-label = Edytor grupy\n\
groups-name-label = Nazwa grupy\n\
groups-name-ph = np. Redaktorzy Antologii\n\
groups-desc-label = Opis\n\
groups-desc-ph = Nad czym współpracuje ta grupa\n\
groups-create-btn = Utwórz grupę\n\
groups-invite-btn = Zaproś do grupy\n\
groups-set-member-btn = Ustaw jako członka\n\
groups-assign-mgr-btn = Przypisz menedżera\n\
invites-title = Zaproszenia i polecenia\n\
invites-sub = Zaproś pisarzy i śledź polecone rejestracje.\n\
invites-create-btn = Utwórz link zaproszenia\n\
invites-th-scope = Zakres zaproszenia\n\
invites-th-link = Link zaproszenia\n\
invites-th-uses = Użycia\n\
invites-th-created = Utworzono\n\
invites-scope-general = Ogólne zaproszenie ExoNovel\n\
wallet-tab-title = Połączone portfele i metody płatności\n\
wallet-tab-sub = Wybierz sposób odbierania płatności od czytelników i opłacania planów synchronizacji osobistej w lokalnych walutach, kartami oraz kryptowalutami.\n\
wallet-selected-rail-title = Aktywna ścieżka rozliczeń\n\
wallet-selected-rail-desc = Używana przy aktywacji planów synchronizacji osobistej oraz wypłatach dla autorów.\n\
wallet-card-desc = Karty, Apple Pay, Google Pay oraz lokalne bramki walutowe.\n\
wallet-solana-desc = Natychmiastowe wypłaty w stablecoinach i odblokowania czytelników.\n\
wallet-btc-desc = Tanie faktury Lightning dla globalnych mikroodblokowań i subskrypcji.\n\
ledger-wallet-title = Transakcje portfela i paragony\n\
ledger-wallet-sub = Pełny rejestr przychodów autora, paragonów czytelników i opłat za plan synchronizacji.\n\
paywalls-title = Paywalle autorów i dostęp czytelników\n\
paywalls-sub = Zdecyduj, które historie, rozdziały i elementy świata pozostają darmowe dla wszystkich, a które odblokowują się za opłatą — z automatycznym podziałem przychodów dla współautorów.\n\
paywalls-free-tier-title = Darmowe rozdziały publiczne\n\
paywalls-free-tier-desc = Udostępniaj pierwsze rozdziały lub całe książki bezpłatnie, aby nowi czytelnicy mogli poznać Twoją twórczość.\n\
paywalls-unlock-title = Paywalle dla opowieści i rozdziałów\n\
paywalls-unlock-desc = Ustalaj ceny odblokowania w preferowanej walucie dla kart, płatności mobilnych i portfeli krypto.\n\
paywalls-splits-title = Podział przychodów współautorów\n\
paywalls-splits-desc = Gdy piszecie wspólnie w grupie, przychody z paywalli trafiają przejrzyście do każdego przypisanego twórcy.\n\
paywalls-ledger-title = Odblokowania czytelników i aktywność paywalli\n\
paywalls-ledger-sub = Dzieła odblokowane przez Ciebie jako czytelnika oraz płatne odblokowania w Twoich historiach.\n\
paywalls-empty = Brak jeszcze zarejestrowanych sprzedaży paywalli ani odblokowań czytelników.\n\
plans-title = Dostęp do pisania i plany synchronizacji własnych urządzeń\n\
plans-sub = Pisanie w przeglądarce jest darmowe, pisanie na własnych urządzeniach jest darmowe, a synchronizacja współpracy między pisarzami jest zawsze darmowa — nawet gdy jedna osoba jest w sieci, a druga offline na urządzeniu. Płatny plan służy wyłącznie do synchronizacji Twoich własnych szkiców między urządzeniem a profilem w sieci.\n\
plans-payment-rail-note = Wybrana metoda płatności\n\
plans-change-wallet-btn = Konfiguruj w Portfelu →\n\
plans-free-title = Sieć, urządzenie i współpraca (Darmowe)\n\
plans-free-desc = Darmowe pisanie w przeglądarce, darmowe pisanie lokalne na urządzeniach oraz darmowa synchronizacja współpracy z zespołem.\n\
wallet-peer-prefix = Docelowy węzeł usługi ExoNovel:\n\
wallet-currency-label = Lokalna waluta\n\
wallet-rail-label = Domyślna metoda płatności (AOTA)\n\
wallet-cur-usd = USD (9 $/mies. · 90 $/rok)\n\
wallet-cur-eur = EUR (9 €/mies. · 90 €/rok)\n\
wallet-cur-gbp = GBP (7 £/mies. · 70 £/rok)\n\
wallet-cur-pln = PLN (36 zł/mies. · 360 zł/rok)\n\
wallet-cur-ngn = NGN (13 500 ₦/mies. · 135 000 ₦/rok)\n\
wallet-cur-kes = KES (1200 KSh/mies. · 12 000 KSh/rok)\n\
wallet-cur-inr = INR (750 ₹/mies. · 7500 ₹/rok)\n\
wallet-cur-brl = BRL (45 R$/mies. · 450 R$/rok)\n\
wallet-rail-card = Karta / lokalna bramka walutowa\n\
wallet-rail-solana = Solana Pay / USDC\n\
wallet-rail-btc = Bitcoin Lightning (L402)\n\
wallet-rail-eth = Base / Arbitrum / Ethereum\n\
wallet-rail-mobile = Płatność mobilna / PIX / UPI\n\
wallet-free-current = Obecny plan (Darmowy)\n\
wallet-free-switch = Przełącz na darmowy\n\
wallet-mo-title = Synchronizacja osobista miesięczna (ekw. 9 $/mies.)\n\
wallet-mo-desc = Szyfrowana synchronizacja łącząca Twoje własne szkice offline z Twoim profilem w przeglądarce.\n\
wallet-mo-active = Aktywny plan miesięczny\n\
wallet-mo-activate = Aktywuj 9 $/miesiąc\n\
wallet-yr-title = Synchronizacja osobista roczna (ekw. 90 $/rok)\n\
wallet-yr-desc = 2 miesiące gratis · pełna szyfrowana synchronizacja między Twoimi urządzeniami a profilem w sieci.\n\
wallet-yr-active = Aktywny plan roczny\n\
wallet-yr-activate = Aktywuj 90 $/rok\n\
ledger-th-type = Typ\n\
ledger-th-amount = Kwota\n\
ledger-th-cycle = Cykl\n\
ledger-th-details = Szczegóły\n\
ledger-th-date = Data\n\
personas-title = Pseudonimy i persony\n\
personas-sub = Każda persona posiada własny klucz podprzestrzeni kryptograficznej oraz awatar, łącząc się z Twoją główną tożsamością.\n\
persona-default-writer = Pisarz\n\
persona-use-btn = Użyj\n\
persona-edit-btn = Edytuj personę i awatar\n\
persona-display-ph = Nazwa wyświetlana\n\
persona-handle-ph = Uchwyt (@pseudonim)\n\
persona-bio-ph = Biogram\n\
persona-save-btn = Zapisz personę\n\
persona-create-heading = Utwórz nową personę\n\
persona-composer-label = Edytor persony\n\
persona-display-label = Nazwa wyświetlana persony\n\
persona-handle-label = Uchwyt\n\
persona-bio-label = Biogram\n\
persona-create-btn = Utwórz personę\n\
langs-quick-title = Osobiste języki szybkiego przełączania\n\
langs-quick-sub = Wybierz, które z 29 obsługiwanych języków pojawią się w kompaktowym przełączniku paska górnego i edytorze pisania. Wszystkie 29 pozostają dostępne w Preferencjach.\n\
langs-pinned-ui-label = Przypięte języki interfejsu\n\
langs-pinned-input-label = Przypięte języki wprowadzania tekstu\n\
langs-save-btn = Zapisz listy szybkiego przełączania\n\
activity-notif-title = Powiadomienia o aktywności, współpracy i sprzedaży\n\
activity-notif-sub = Wybierz, które zdarzenia persony, zaproszenia do grup, przychody z paywalli i zakupy czytelników mają wysyłać powiadomienia.\n\
notif-earn-title = Przychody autora z paywalli\n\
notif-earn-desc = Powiadomienie, gdy czytelnik odblokuje Twoje płatne dzieła.\n\
notif-purch-title = Zakupy czytelników i paragony\n\
notif-purch-desc = Potwierdzenia zakupu dzieł lub odblokowania paywalli innych autorów.\n\
notif-inv-title = Zaproszenia do grup i zmiany ról\n\
notif-inv-desc = Zaproszenia i przypisania roli Menedżera.\n\
notif-ref-title = Rejestracje z poleceń\n\
notif-ref-desc = Alerty, gdy pisarz dołączy przez Twój osobisty link zaproszenia.\n\
notif-collab-title = Uwagi współautorów i współpracowników\n\
notif-collab-desc = Komentarze i odpowiedzi w udostępnionych dziełach.\n\
notif-fork-title = Wzmianki i odgałęzienia (forki)\n\
notif-fork-desc = Alerty, gdy Twoje elementy fabuły zostaną rozwidlone z zachowaniem autorstwa.\n\
activity-notif-save-btn = Zapisz reguły powiadomień o aktywności\n\
profile-toast-persona-created = Utworzono personę\n\
profile-toast-persona-updated = Zaktualizowano personę\n\
profile-msg-img-size = Obraz może mieć maksymalnie 2 MB.\n\
profile-toast-avatar-done = Przetworzono awatar persony\n\
profile-toast-group-created = Utworzono grupę\n\
profile-toast-role-updated = Zaktualizowano rolę członka\n\
profile-toast-invite-copied = Skopiowano link zaproszenia\n\
profile-toast-plan-updated = Zaktualizowano plan synchronizacji\n\
profile-toast-langs-saved = Zapisano szybkie przełączniki języków\n\
profile-toast-notif-saved = Zapisano preferencje powiadomień o aktywności\n\
cms-all = Wszystkie\n\
cms-on-hold = Wstrzymane\n\
cms-ready = Gotowe\n\
cms-accepted = Zaakceptowane\n\
cms-proposed = Proponowane\n\
cms-search-placeholder = Szukaj ADR...\n\
cms-sidebar-title = MADR 4.0.0 CMS\n\
cms-records-count-suffix = rekordów\n\
cms-hide = Ukryj\n\
cms-show-explorer = Pokaż eksplorator\n\
cms-read-adr = Czytaj ADR\n\
cms-new-madr = + Nowy MADR 4.0\n\
cms-tooling-proposals = Narzędzia i propozycje\n\
cms-inference-runtimes = Środowiska wnioskowania AI\n\
cms-owner-tasks = Zadania właściciela i operacje\n\
cms-connect-google-docs = Połącz Dokumenty Google\n\
cms-backup-drive = Kopia zapasowa na Dysk\n\
cms-backup-title = Utwórz kopię zapasową wszystkich dokumentów ExoNovel na Dysku Google\n\
cms-backup-dest-default = Miejsce docelowe: ExoNovel Docs/\n\
cms-backup-dest-prefix = Miejsce docelowe: ExoNovel Docs/ na\n\
cms-backup-fmt-gdoc = Dokumenty Google (edytowalne)\n\
cms-backup-fmt-md = Markdown (.md)\n\
cms-backup-start = Rozpocznij kopię zapasową\n\
cms-drive-synced-suffix = jest zsynchronizowany na Dysku Google.\n\
cms-open-drive-btn = Otwórz w Dokumentach / Dysku Google ↗\n\
cms-select-adr-prompt = Wybierz rekord z eksploratora po lewej stronie.\n\
cms-words-suffix = słów\n\
cms-min-read-suffix = min czytania\n\
cms-trans-canonical = Kanoniczny (EN)\n\
cms-trans-translated = Przetłumaczono\n\
cms-trans-fallback-en = Wersja angielska (brak tłumaczenia)\n\
cms-save-gdoc-btn = Zapisz w Dokumentach Google\n\
cms-save-md-btn = Zapisz .md na Dysku\n\
cms-edit-adr-btn = Edytuj w MADR CMS\n\
cms-deps-title = Zależności narzędziowe i blokery\n\
cms-deps-empty = Brak blokujących zależności narzędziowych przypisanych do tego ADR.\n\
cms-deps-installed = zainstalowano\n\
cms-deps-latest = najnowsza\n\
cms-deps-req = wymagana\n\
cms-deps-constraint-ph = >=0.4.0\n\
cms-deps-cap-ph = Blokująca funkcja...\n\
cms-deps-satisfied = Spełnione\n\
cms-deps-link-btn = Powiąż zależność\n\
cms-editor-title-edit = Edytuj rekord MADR 4.0.0\n\
cms-editor-title-new = Utwórz nowy rekord MADR 4.0.0\n\
cms-editor-context-label = Edytor MADR\n\
cms-field-title = Tytuł\n\
cms-field-rel-path = Ścieżka względna (np. docs/adr/010-short.md)\n\
cms-field-status = Status\n\
cms-field-makers = Decydenci\n\
cms-field-context = Kontekst i opis problemu\n\
cms-field-drivers = Czynniki decyzyjne\n\
cms-field-outcome = Wynik decyzji\n\
cms-field-raw-md = Pełny Markdown (opcjonalne nadpisanie)\n\
cms-save-adr-btn = Zapisz ADR\n\
cms-status-opt-proposed = proposed\n\
cms-status-opt-on-hold = on-hold (oczekuje na narzędzia)\n\
cms-status-opt-ready = ready-to-build\n\
cms-status-opt-accepted = accepted\n\
cms-status-opt-deprecated = deprecated\n\
cms-status-opt-superseded = superseded\n\
cms-status-opt-rejected = rejected\n\
cms-on-this-page = Na tej stronie\n\
cms-tooling-catalog-title = Katalog narzędzi i skaner wydań\n\
cms-tooling-catalog-sub = Deterministyczny monitoring wersji crates.io powiązany z rekordami decyzji architektonicznych.\n\
cms-run-scan = Uruchom skan crates.io\n\
cms-th-tool = Narzędzie\n\
cms-th-installed = Zainstalowano\n\
cms-th-latest = Najnowsza\n\
cms-th-purpose = Przeznaczenie\n\
cms-add-tool-heading = Śledź kolejną zależność / narzędzie\n\
cms-add-tool-key-ph = Klucz (np. reqwest)\n\
cms-add-tool-name-ph = Nazwa wyświetlana\n\
cms-add-tool-pkg-ph = Pakiet crates.io\n\
cms-add-tool-ver-ph = Zainstalowana wersja\n\
cms-add-tool-purpose-ph = Dlaczego ExoNovel od tego zależy\n\
cms-add-tool-btn = Dodaj narzędzie\n\
cms-ai-providers-title = Dostawcy wnioskowania AI i aktywne modele\n\
cms-ai-providers-sub = Skonfigurowane lokalne środowisko wykonawcze CLI oraz zweryfikowane aktywne modele w tym środowisku.\n\
cms-refresh-models = Odśwież aktywne modele\n\
cms-ai-empty = Brak aktywnych dostawców wnioskowania skonfigurowanych w tym środowisku.\n\
cms-model-deprecated = Wycofany\n\
cms-model-cooldown = Wstrzymany\n\
cms-model-active = Aktywny\n\
cms-maturity-proposals-title = Propozycje dojrzałości\n\
cms-maturity-proposals-sub = Generuj rekomendacje aktualizacji dla śledzonych zależności oraz akceptuj lub odrzucaj propozycje.\n\
cms-generate-proposal = Generuj notatkę propozycji\n\
cms-proposals-empty = Brak notatek propozycji w wybranym filtrze. Wybierz narzędzie powyżej i wygeneruj propozycję.\n\
cms-proposal-filter-all = Wszystkie\n\
cms-proposal-filter-open = Otwarte\n\
cms-proposal-filter-accepted = Zaakceptowane\n\
cms-proposal-filter-dismissed = Odrzucone\n\
cms-proposal-status-prefix = Status:\n\
cms-proposal-accept = Zaakceptuj propozycję\n\
cms-proposal-dismiss = Odrzuć\n\
cms-proposal-reopen = Otwórz ponownie\n\
cms-proposal-rationale = Diagnostyka / Uzasadnienie:\n\
cms-owner-tasks-title = Zewnętrzna lista zadań właściciela i operacji\n\
cms-owner-tasks-sub = Zadania wymagające dostępu do zewnętrznych konsol lub konfiguracji serwera produkcyjnego. Zaznacz po wykonaniu, aby zapisać stan ukończenia w bazie danych.\n\
cms-task-done = Gotowe\n\
cms-task-pending = Oczekuje\n\
cms-msg-redirecting-google = Przekierowywanie do ekranu zgody Google...\n\
cms-msg-saving-gdoc = Zapisywanie aktywnego ADR do Dokumentów Google...\n\
cms-msg-saving-md = Tworzenie kopii aktywnego ADR jako .md na Dysku Google...\n\
cms-msg-save-failed = Zapis nie powiódł się.\n\
cms-toast-saved-drive = Zapisano w Dokumentach / Dysku Google\n\
cms-msg-backing-up-archive = Tworzenie kopii pełnego archiwum ExoNovel Docs i ADR na Dysku Google (to może chwilę potrwać)...\n\
cms-msg-archive-failed = Kopia archiwum nie powiodła się.\n\
cms-toast-archive-backed-up = Zapisano kopię archiwum na Dysku Google\n\
cms-toast-task-done = Zadanie oznaczone jako ukończone\n\
cms-toast-task-reopened = Zadanie otwarte ponownie\n\
cms-toast-task-failed = Nie udało się zaktualizować zadania\n\
cms-msg-saving-adr = Zapisywanie rekordu MADR 4.0.0...\n\
cms-msg-missing-token = Brak tokenu sesji.\n\
cms-toast-adr-saved = Zapisano ADR w CMS\n\
cms-toast-dep-linked = Powiązano zależność narzędzia z ADR\n\
cms-msg-scanning-crates = Wykonywanie deterministycznego skanu wydań crates.io...\n\
cms-toast-scan-done = Zaktualizowano katalog wydań crates.io\n\
cms-msg-scan-failed = Skanowanie nie powiodło się.\n\
cms-msg-refreshing-models = Odświeżanie aktywnych modeli w skonfigurowanych środowiskach wnioskowania...\n\
cms-toast-models-refreshed = Odświeżono aktywne modele dostawców\n\
cms-msg-generating-proposal = Uruchamianie analizy propozycji aktywnego wnioskowania...\n\
cms-toast-proposal-generated = Wygenerowano notatkę propozycji dojrzałości\n\
cms-toast-proposal-updated = Zaktualizowano status propozycji\n\
cms-toast-tool-added = Dodano narzędzie do katalogu dojrzałości\n\
cms-toast-copied = Skopiowano tekst do schowka\n\
nav-group-account = Konto\n\
nav-group-workspace = Przestrzeń robocza i powiadomienia\n\
nav-group-admin = Administracja\n\
nav-group-writing = Pisanie i współpraca\n\
nav-group-commerce = Rozliczenia i dostęp czytelników\n\
nav-group-personalization = Personalizacja\n\
settings-resize-aria = Zmień szerokość paska nawigacji ustawień\n",
        "de" => "\
nav-sign-in = Anmelden\n\
nav-toggle-menu = Navigationsmenü umschalten\n\
nav-privacy = Datenschutzerklärung\n\
nav-terms = Nutzungsbedingungen\n\
live-connection-title = Globale Echtzeit-Stream-Verbindung\n\
ui-lang-aria = Ausgabesprache der App\n\
input-lang-aria = Eingabesprache beim Schreiben\n\
notifications-aria = Benachrichtigungen\n\
switch-persona = Persona wechseln\n\
create-personas-prompt = Erstelle Personas in deinem Profil\n\
default-persona-label = Standard-Persona\n\
active-badge = Aktiv\n\
mark-all-read = Alle als gelesen markieren\n\
mark-read = Als gelesen markieren\n\
no-notifications = Noch keine Benachrichtigungen.\n\
refresh-label = Aktualisieren\n\
cancel-label = Abbrechen\n\
dismiss-label = Schließen\n\
save-changes-short = Speichern\n\
confirm-label = Bestätigen\n\
copy-text-label = Text kopieren\n\
pen-name-label = Künstlername:\n\
pending-did-label = Ausstehende did:peer\n\
workspace-desc = Dein Schreibtisch hält alles, woran du gerade arbeitest, griffbereit — ob du deine erste Geschichte für die Schule gliederst, mit Freunden Figuren entwickelst oder allein Gedichte schreibst. Jede Figur, Zeitleiste, Szene und Notiz bleibt mit deinem Künstlernamen verknüpft, damit du Ideen frei teilen, gemeinsam lernen und deine Werke immer als deine eigenen behalten kannst.\n\
btn-profile-billing = Profil, Personas & Abrechnung\n\
btn-connected-apps = Verbundene Apps & Google Docs\n\
btn-open-cms = MADR Docs CMS öffnen\n\
workspace-key-label = Aktiver Persona-Schlüssel (did:peer)\n\
choose-image-btn = Profilbild wählen\n\
persona-choose-avatar-btn = Persona-Bild wählen\n\
desk-start-eyebrow = Neues Projekt starten\n\
desk-start-title = Was möchtest du heute schreiben?\n\
desk-starter-novel-hint = Bereit auf deinem Regal: Handlungsübersicht · Figurenkarten · Kapitel-1-Entwurf · Welt-Zeitleiste.\n\
desk-starter-article-hint = Bereit auf deinem Regal: Einstiegsthese · Abschnittsgliederung · Quellennotizen · Aufsatz-Entwurf.\n\
desk-starter-short-hint = Bereit auf deinem Regal: Einstiegshaken · Wendepunkt · Figuren-Moment · Szenenentwurf.\n\
desk-starter-script-hint = Bereit auf deinem Regal: Szenenkopf · Besetzung & Stimmen · Beat Sheet · Drehbuchseiten.\n\
desk-starter-poetry-hint = Bereit auf deinem Regal: Strophenentwurf · Rhythmusnotizen · Band-Reihenfolge · Widmung.\n\
desk-blocks-eyebrow = Geschichten-Bausteine\n\
desk-blocks-title = Dein Regal voller Bausteine\n\
desk-block-chars-title = Figuren\n\
desk-block-chars-desc = Behalte Heldinnen, Gegenspieler und Vorgeschichten im Blick — oder hole eine Figur von Freunden mit klarer Urhebernennung in deine Szene.\n\
desk-block-plots-title = Handlungen\n\
desk-block-plots-desc = Skizziere Wendepunkte, Kapitel und Überraschungen, damit deine Geschichte rund vom Anfang bis zum Schluss fließt.\n\
desk-block-timelines-title = Zeitleisten\n\
desk-block-timelines-desc = Ordne, was zuerst, danach oder vor Jahren geschah — ideal für Krimis, Trilogien oder gemeinsame Schulprojekte.\n\
desk-block-themes-title = Themen & Motive\n\
desk-block-themes-desc = Halte die großen Fragen, Stimmungen und Leitmotive fest, damit alle Co-Autorinnen auf derselben Wellenlänge bleiben.\n\
desk-scratchpad-eyebrow = Notizblock\n\
desk-scratchpad-label = Schneller Ideenblock\n\
desk-scratchpad-title = Halte eine schnelle Idee fest\n\
desk-note-title-label = Arbeitstitel oder Impuls\n\
desk-note-title-ph = z. B. Regen auf den Stufen der Bibliothek...\n\
desk-note-body-label = Erste Zeilen oder Szenenskizze\n\
desk-note-body-ph = Schreibe ein paar Sätze, teste einen Dialog oder wechsle oben deine Schreibsprache...\n\
desk-note-keep-btn = Idee auf dem Schreibtisch anheften\n\
desk-note-clear-btn = Leeren\n\
desk-note-pinned-msg = Idee für diese Sitzung auf deinem Schreibtisch angeheftet\n\
guides-eyebrow = Dear World · Benutzerhandbücher\n\
guides-sidebar-title = Erste Schritte\n\
guides-sidebar-sub = Schritt-für-Schritt-Anleitungen für Schülerinnen und Studierende, die gemeinsames Schreiben auf ExoNovel entdecken.\n\
guides-open-desk-btn = Zum Schreibtisch\n\
guides-signin-btn = Anmelden und loslegen\n\
guides-loading = Dear World Guide wird geöffnet...\n\
guides-empty = Wähle links ein Kapitel aus, um zu beginnen.\n\
toast-did-copied = did:peer-ID kopiert\n\
toast-switched-persona = Persona gewechselt\n\
oauth-connected-google = Google Drive & Google Docs erfolgreich verbunden.\n\
theme-system-label = Systemdesign\n\
theme-light-label = Helles Design\n\
theme-dark-label = Dunkles Design\n\
role-admin = Admin\n\
role-moderator = Moderator\n\
role-user = Benutzer\n\
status-reader = Leser\n\
status-pending = Ausstehend\n\
status-suspended = Gesperrt\n\
status-restricted = Eingeschränkt\n\
status-manager = Manager\n\
status-member = Mitglied\n\
landing-eyebrow = ExoNovel · Web, Desktop & Mobil\n\
landing-title-row1 = Kollaboratives kreatives Schreiben\n\
landing-title-row2 = Bei dem Autorinnen und Autoren ihre Werke selbst besitzen\n\
landing-title-row3 = ... von Grund auf!\n\
landing-lead = ExoNovel ist eine Plattform für kollaboratives kreatives Schreiben und Local-First-Publizieren für Autorinnen, Co-Autoren und Literaturteams. Verfasse Romane, Essays, Kurzgeschichten, Drehbücher und Lyrik gemeinsam und behalte kryptografisches Eigentum an deinen Manuskripten.\n\
landing-cta = Bei ExoNovel anmelden\n\
landing-cap-aria = Plattformfunktionen\n\
landing-prev-slide = Vorherige Folie\n\
landing-next-slide = Nächste Folie\n\
slide-1-tab = Deine Texte, deine Rechte\n\
slide-1-title = Gemeinsam erschaffen und Eigentümer jeder eigenen Idee bleiben\n\
slide-1-p1 = ExoNovel ist ein kreativer Schreibraum, in dem du allein schreiben oder in Echtzeit mit Freundinnen, Co-Autoren und Schreibgruppen zusammenarbeiten kannst. Statt in unübersichtlichen geteilten Dokumenten den Überblick zu verlieren, wer was verfasst hat, bleiben jedes Figurenprofil, jeder Handlungsbogen, jede Zeitleiste, jedes Thema und jede Szene eindeutig und sicher der Person zugeordnet, die sie erschaffen hat — einschließlich deiner Kommentare zu den Werken anderer.\n\
slide-1-p2 = Zusammenarbeit kostet dich also niemals dein eigenes Werk. Du entscheidest, was privat bleibt, was deine Gruppe gemeinsam bearbeiten darf und was andere Schreibende mit deiner Erlaubnis in ihre eigenen Geschichten übernehmen und weiterspinnen können — mit deiner völlig unabhängigen, dezentralen Identität dauerhaft an deinem Werk.\n\
slide-2-tab = Ein Kopf, viele Personas\n\
slide-2-title = Eine Identität, so viele Personas, wie du möchtest\n\
slide-2-p1 = Eine Sci-Fi-Geschichte mit Freunden zu schreiben, Lyrik unter einem Pseudonym zu teilen und Sachtexte unter deinem echten Namen zu verfassen, sollte dich nicht zwingen, drei verschiedene Konten zu verwalten. ExoNovel unterstützt beliebig viele Personas — jede mit eigenem Anzeigenamen bzw. Handle, Bio und Avatar, zugleich als Publikations-Proxy (du kannst direkt als Persona veröffentlichen) sowie mit eigenem Zugang zu deiner Leser- und Mitwirkenden-Community, sodass du sogar als deine Persona über eigene und fremde Texte chatten kannst. Niemand auf ExoNovel muss seine wahre Identität öffentlich preisgeben, darf es aber jederzeit — alles unter einem einzigen, völlig unabhängigen Login.\n\
slide-2-p2 = Wechsle mit einem Klick sofort in einen anderen Persona-Arbeitsbereich mit eigenen Projekten, Gruppenchats und Genres. Mitwirkende und Lesende sehen genau den Persona-Anzeigenamen, den du für den jeweiligen Bereich gewählt hast, während alles, was du über alle Personas hinweg schreibst, dank der sicher dezentralen P2P-ID deiner Identität sauber organisiert und dauerhaft rechtmäßig dein Eigentum bleibt.\n\
slide-3-tab = Schreiben, wo du bist\n\
slide-3-title = Online und offline — alle Geräte im Einklang\n\
slide-3-p1 = Schreibe direkt im Browser, wann immer du online bist, oder arbeite ohne Internetverbindung weiter mit unseren kostenlosen Apps für Linux, macOS, Windows, Android und iOS. Unsere kostenlosen Apps halten dich mit deiner Web-Arbeit verbunden, und gegen eine kleine monatliche Gebühr (oder noch günstiger im Jahresabo) sind deine Online- und lokalen Werke jederzeit gemeinsam in deinem einen virtuellen Schreibzuhause griffbereit.\n\
slide-3-p2 = Beginne unterwegs auf dem Smartphone eine Szene, feile später am Laptop an einem Kapitel und synchronisiere deine Entwürfe bei Bedarf manuell zwischen Offline-Geräten und Web — oder aktiviere unsere automatische Synchronisation, damit jeder Entwurf auf allen Bildschirmen aktuell bleibt.\n\
slide-4-tab = Verbinde auf deine Art\n\
slide-4-title = Verknüpfe deine Tools, behalte volle Freiheit\n\
slide-4-p1 = Deine Texte sollten niemals in einer einzigen Plattform eingesperrt sein. Melde dich so an, wie es dir am besten passt, schreibe in den Formaten, die zu deinem Projekt gehören, und verknüpfe externe Cloud-Speicher, Schreibtools und Workflows für Backups, schnelle Exporte, Veröffentlichungen, plattformübergreifende Projekte oder was immer dein kreativer Prozess verlangt.\n\
slide-4-p2 = Alles in ExoNovel greift ineinander, ohne dich festzunageln. Füge Verbindungen hinzu, wann immer sie helfen, kombiniere Werkzeuge, wenn deine Projekte wachsen, und trenne jeden Dienst jederzeit wieder — bei vollem Zugriff auf dein Konto, deine Personas und deine Werke.\n\
landing-modes-title = Im Web, auf deinem Gerät und gemeinsam synchronisiert\n\
landing-modes-sub = Schreibe kostenlos im Webbrowser und in unseren nativen Geräte-Apps. Zusammenarbeit zwischen Schreibenden synchronisiert automatisch und kostenlos zwischen Web und Gerät — nur für die automatische Synchronisierung deiner eigenen persönlichen Entwürfe zwischen Web und Geräten fällt ein kleiner Monats- oder Jahresbetrag an.\n\
mode-free-badge = Dauerhaft kostenlos\n\
mode-free-title = Im Web & auf deinem Gerät\n\
mode-free-desc = Schreibe, lade Co-Autorinnen ein und arbeite zusammen im Browser sowie in unseren kostenlosen Linux-, macOS-, Windows-, Android- und iOS-Apps — inklusive kostenloser automatischer Team-Synchronisation zwischen Web und Geräten.\n\
mode-monthly-badge = Persönliche Sync\n\
mode-monthly-title = Persönliche Sync — 9 $ / Monat\n\
mode-monthly-desc = Halte deine eigenen Entwürfe im Web und auf deinen Offline-Geräten mit Ende-zu-Ende-Verschlüsselung automatisch synchron (9 $/Monat oder Gegenwert in Landeswährung).\n\
mode-annual-badge = Jährliche persönliche Sync\n\
mode-annual-title = Persönliche Sync — 90 $ / Jahr\n\
mode-annual-desc = Volle automatische Synchronisierung zwischen deinem eigenen Web-Profil und deinen persönlichen Geräten (effektiv 7,50 $/Monat in Landeswährung — zwei Monate gratis gegenüber dem Monatsplan).\n\
landing-formats-title = Format wählen und direkt losschreiben\n\
landing-formats-sub = Ob eigener Roman, Schulaufsatz oder Drehbuch für einen Kurzfilm — starte mit einem klaren Layout, das zu deiner Art zu schreiben passt.\n\
format-novel-title = Romane\n\
format-novel-desc = Baue ganze Welten von Grund auf. Behalte Figuren, Plot-Twists und Kapitel an einem Ort, damit große Ideen übersichtlich bleiben.\n\
format-article-title = Artikel & Sachbücher\n\
format-article-desc = Mach aus Ideen und Recherche starke Essays, Blogposts oder Schularbeiten mit klaren Gliederungen und sauberen Quellen.\n\
format-short-title = Kurzgeschichten\n\
format-short-desc = Triff den Einstieg, den Twist und das Ende auf den Punkt — ideal für Flash Fiction, einzelne Stories oder eigene Sammlungen.\n\
format-screenplay-title = Drehbücher & Skripte\n\
format-screenplay-desc = Schreibe Szenen und Dialoge wie in echten Film- oder Videoskripten — sauber formatiert, während du tippst.\n\
format-poetry-title = Lyrik\n\
format-poetry-desc = Gestalte Rhythmus, Strophen und Zeilenumbrüche genau so, wie du sie setzt — vom einzelnen Gedicht bis zum eigenen Band.\n\
signin-back-home = ← Zurück\n\
signin-title = Bei ExoNovel anmelden\n\
signin-subtitle = Authentifiziere dich, um deine maßgebliche did:peer-Identität aufzulösen und deinen Arbeitsbereich zu öffnen.\n\
signin-checking-session = Bestehende Sitzung wird geprüft...\n\
signin-invite-badge = Persönliche Einladung\n\
signin-invited-group = hat dich in diese Gruppe auf ExoNovel eingeladen:\n\
signin-invited-platform = hat dich zu ExoNovel eingeladen.\n\
signin-create-heading = Konto erstellen\n\
signin-continue-heading = Weiter zu ExoNovel\n\
signin-or-email = oder per E-Mail\n\
signin-name-label = Name\n\
signin-email-label = E-Mail\n\
signin-password-label = Passwort\n\
signin-authenticating = Authentifizierung läuft...\n\
signin-create-btn = Konto erstellen\n\
signin-submit-btn = Anmelden\n\
signin-have-account = Bereits ein Konto? Anmelden\n\
signin-need-account = Noch kein Konto? Konto erstellen\n\
signin-auth-failed = Authentifizierung fehlgeschlagen.\n\
footer-brand-prefix = ExoNovel, von\n\
privacy-page-title = Datenschutzerklärung — ExoNovel\n\
privacy-effective = Inkrafttreten & letzte Aktualisierung: 24. November 2026\n\
privacy-heading = ExoNovel Datenschutzerklärung\n\
privacy-intro = ExoNovel (https://exonovel.exosystems.net) ist eine Plattform für kollaboratives kreatives Schreiben und Local-First-Publizieren, betrieben von Exosystems (https://exosystems.net). Diese Erklärung beschreibt die Erhebung, Nutzung, Speicherung und Löschung personenbezogener Daten sowie von Google-Nutzerdaten.\n\
privacy-s1-title = 1. Welche Daten wir erheben\n\
privacy-s1-intro = Wir erheben nur die Daten, die für die Kontosicherheit, den kollaborativen Arbeitsbereich und ausdrücklich aktivierte Funktionen erforderlich sind:\n\
privacy-s1-google-label = Google-Anmeldedaten (openid, email, profile): \n\
privacy-s1-google-body = Bei der Anmeldung mit Google erhalten wir Ihre verifizierte E-Mail-Adresse, den Verifizierungsstatus, den Anzeigenamen, Vor- und Nachnamen, die Profilbild-URL, das Gebietsschema sowie die Google-Konto-ID.\n\
privacy-s1-drive-label = Optionale verbundene Google-Drive-Daten: \n\
privacy-s1-drive-body = Die primäre Anmeldung fordert niemals Google-Drive-Berechtigungen an. Erst wenn Sie Google Drive explizit in den Präferenzen oder der Dokumentation verbinden, darf ExoNovel Ordner auflisten, Markdown/ADR-Sicherungen hochladen sowie ausgewählte Dateien wiederherstellen oder in den Papierkorb verschieben.\n\
privacy-s1-profile-label = Weitere Anmelde- & Autorenprofilinformationen: \n\
privacy-s1-profile-body = Bei GitHub- oder E-Mail-Anmeldung speichern wir Anzeigenamen, Künstlernamen, Biografie, Website, Standort, bevorzugtes Format, UI- und Eingabesprachen, Zeitzone, Benachrichtigungsregeln, Avatare (64px, 256px, 1024px) sowie den öffentlichen Schlüssel / did:peer.\n\
privacy-s1-works-label = Manuskripte & Zusammenarbeitsdaten: \n\
privacy-s1-works-body = Kreative Werke, Story-Bibeln, Personas, Gruppenmitgliedschaften, Empfehlungen und Einladungen, die Sie erstellen oder teilen.\n\
privacy-s1-sec-label = Sicherheit & technische Diagnosedaten: \n\
privacy-s1-sec-body = Anmeldezeitpunkte, IP-Adresse, Browser-User-Agent und Fehlerprotokolle ausschließlich zum Schutz vor Missbrauch und für den zuverlässigen Betrieb.\n\
privacy-s2-title = 2. Cookies & Browser-Speicher\n\
privacy-s2-body = ExoNovel nutzt ausschließlich technisch notwendige HttpOnly-Authentifizierungs-Cookies sowie temporären Sitzungsspeicher des Browsers zur Erhaltung ausstehender Einladungscodes bei Anmelde-Weiterleitungen. Es werden keine Werbe- oder Tracking-Cookies verwendet.\n\
privacy-s3-title = 3. Wie wir Ihre Daten verwenden\n\
privacy-s3-li1 = Authentifizierung deiner Sitzung und Auflösung deiner did:peer-Identität im Web, auf dem Desktop und mobil.\n\
privacy-s3-li2 = Anzeige Ihrer gewählten Persona, des Künstlernamens, des Avatars und der Sprachpräferenzen (inkl. RTL).\n\
privacy-s3-li3 = Betrieb gemeinsamer Schreibgruppen, Autorenzuordnung und Einladungen.\n\
privacy-s3-li4 = Ausführung benutzergesteuerter Sicherungen und Exporte in Ihr verbundenes Google Drive.\n\
privacy-s3-li5 = Versand aktivierter Transaktions-, Sicherheits- und Benachrichtigungs-E-Mails (von notes@exosystems.net, noreply@exosystems.net oder support@exosystems.net).\n\
privacy-s3-li6 = Schutz vor Betrug, unbefugtem Zugriff und Verstößen gegen unsere Nutzungsbedingungen.\n\
privacy-s4-title = 4. Google API Services User Data Policy (Limited Use) & Verbot von AI NCII\n\
privacy-s4-intro = Die Verarbeitung und Weitergabe von Daten aus Google APIs erfolgt strikt gemäß der Google API Services User Data Policy einschließlich der Limited-Use-Anforderungen.\n\
privacy-s4-li1-label = Keine Werbung & kein Datenhandel: \n\
privacy-s4-li1-body = Wir verkaufen oder vermieten weder Google-Nutzerdaten noch Ihre Texte an Datenbroker oder Werbenetzwerke.\n\
privacy-s4-li2-label = Kein Training allgemeiner KI-Modelle: \n\
privacy-s4-li2-body = ExoNovel nutzt keine Google-Nutzerdaten, Drive-Dateien oder private Manuskripte zum Training allgemeiner KI- oder ML-Modelle.\n\
privacy-s4-li3-label = Keine Bildgenerierung & Nulltoleranz für AI NCII: \n\
privacy-s4-li3-body = ExoNovel ist eine textbasierte Plattform. Sie generiert weder Bilder noch Video oder Audio und untersagt strikt jede Nutzung für AI NCII.\n\
privacy-s4-li4-label = Eingeschränkter Mitarbeiterzugriff: \n\
privacy-s4-li4-body = Exosystems-Mitarbeitende lesen keine privaten Drive-Dateien oder Manuskripte, außer bei ausdrücklicher Support-Einwilligung, Sicherheitsvorfällen oder gesetzlicher Pflicht.\n\
privacy-s5-title = 5. Datenspeicherung, Sicherheit & Kryptografischer Schutz\n\
privacy-s5-body = Der gesamte Datenverkehr zu ExoNovel ist per HTTPS/TLS verschlüsselt. Gehostete Webdaten und Kontostatus werden in verwalteten PostgreSQL-Datenbanken und verwalteten Authentifizierungsdiensten auf einer Enterprise-Cloud-Infrastruktur gespeichert, ergänzt durch transaktionalen E-Mail-Versand. Optionale OAuth-Tokens verbundener externer Apps werden serverseitig durch dedizierte Schlüsselverwaltungs-Verschlüsselung geschützt und von den primären Anmeldedaten isoliert. Auf nativen Geräten mit Peer-to-Peer-Willow- und iroh-Relay-Synchronisierung werden Inhalte Ende-zu-Ende auf dem Gerät verschlüsselt.\n\
privacy-s6-title = 6. Widerruf, Export & Löschung Ihrer Daten\n\
privacy-s6-li1-label = Google-Zugriff jederzeit trennen & widerrufen: \n\
privacy-s6-li1-body = Sie können Google Drive unter Präferenzen -> Verbundene Apps trennen (sofortiger Token-Widerruf über https://oauth2.googleapis.com/revoke) oder jederzeit unter https://myaccount.google.com/permissions widerrufen.\n\
privacy-s6-li2-label = Export- & Löschanfragen: \n\
privacy-s6-li2-body = Fordern Sie einen Export oder die dauerhafte Löschung Ihres Kontos per E-Mail an support@exosystems.net oder exonomyapp@gmail.com an. Verifizierte Anträge werden innerhalb von 30 Tagen abgeschlossen.\n\
privacy-s7-title = 7. Aufbewahrung, Jugendschutz & Aktualisierungen\n\
privacy-s7-body = Wir speichern Kontodaten nur solange das Konto aktiv ist oder gesetzliche Pflichten bestehen. ExoNovel richtet sich nicht an Kinder unter 13 Jahren (bzw. 16 Jahren nach lokalem Recht). Wesentliche Änderungen werden hier veröffentlicht.\n\
privacy-s8-title = 8. Kontakt\n\
privacy-s8-body = Bei Fragen zum Datenschutz oder zu Google-Nutzerdaten wenden Sie sich an support@exosystems.net oder exonomyapp@gmail.com.\n\
terms-page-title = Nutzungsbedingungen — ExoNovel\n\
terms-effective = Inkrafttreten & letzte Aktualisierung: 24. November 2026\n\
terms-heading = ExoNovel Nutzungsbedingungen\n\
terms-intro = Mit dem Zugriff auf ExoNovel (https://exonovel.exosystems.net), betrieben von Exosystems (https://exosystems.net), akzeptieren Sie diese Nutzungsbedingungen und unsere Datenschutzerklärung (/privacy).\n\
terms-s1-title = 1. Eigentum an Ihren kreativen Werken\n\
terms-s1-body = Sie behalten das vollständige Eigentum und Urheberrecht an allen Manuskripten, Figuren, Notizen, Drehbüchern und Gedichten. Sie gewähren ExoNovel lediglich die technisch notwendige Lizenz zur Speicherung, Darstellung und Synchronisierung gemäß Ihren Präferenzen.\n\
terms-s2-title = 2. Konten, Schlüssel & zulässige Nutzung\n\
terms-s2-body = Sie sind für den Schutz Ihrer Zugangsdaten und did:peer-Schlüssel verantwortlich und müssen mindestens 13 Jahre alt sein. Eingriffe in die Plattformsicherheit, unbefugter Zugriff oder Spam sind untersagt.\n\
terms-s3-title = 3. Verbotene Inhalte & Google-APIs-Compliance\n\
terms-s3-intro = ExoNovel ist eine textbasierte Schreibplattform ohne Bild-, Video- oder Audio-Generierung. Wir setzen die Google APIs Terms of Service strikt durch:\n\
terms-s3-li1-label = Striktes Verbot von AI NCII: \n\
terms-s3-li1-body = Sie dürfen ExoNovel oder verbundene Google APIs nicht zur Erstellung, Speicherung oder Verbreitung von AI NCII oder Deepfakes nutzen.\n\
terms-s3-li2-label = Rechtswidrige & missbräuchliche Inhalte: \n\
terms-s3-li2-body = CSAM, nicht einvernehmliche intime Darstellungen, Belästigung, Hassrede und Urheberrechtsverletzungen führen zur sofortigen Kontokündigung.\n\
terms-s4-title = 4. Verbundene externe Apps\n\
terms-s4-body = Optionale Integrationen wie Google Drive werden nur nach Ihrer ausdrücklichen Zustimmung aktiviert und können jederzeit unter Präferenzen -> Verbundene Apps widerrufen werden.\n\
terms-s5-title = 5. Verfügbarkeit, Sync-Tarife & Kontakt\n\
terms-s5-body = ExoNovel bietet Web-Kollaboration, lokale Gerätespeicherung und optionale Tarife für die Synchronisierung persönlicher Geräte. Kontakt: support@exosystems.net oder exonomyapp@gmail.com.\n\
preferences-subtitle = Konto-, Sprach-, Sync-Übersichts- und Benachrichtigungspräferenzen (Käufe, Paywalls und Tarife bleiben im Profil).\n\
preferences-nav-aria = Präferenzen-Bereiche\n\
sec-account = Konto & Souveräne Identität\n\
sec-connected-apps = Verbundene Apps & Google Docs\n\
sec-appearance-langs = Sprachen (29)\n\
sec-sync-storage = Web, Gerät & Team-Sync\n\
sec-notifications = Benachrichtigungskanäle\n\
sec-admin-allowlist = Docs-Zugriffsliste & Konten\n\
account-title = Konto & Souveräne Identität\n\
save-account-btn = Kontopräferenzen speichern\n\
upload-avatar-label = Profilbild hochladen (64px / 256px / 1024px WebP)\n\
root-identity-label = Stammidentität (did:peer)\n\
display-name-label = Anzeigename\n\
pen-name-field-label = Standard-Künstlername\n\
website-label = Website\n\
location-label = Ort & Zeitzone\n\
preferred-format-label = Bevorzugtes Schreibformat\n\
timezone-label = Zeitzone\n\
author-bio-label = Autorenbiografie\n\
format-opt-novel = Romane\n\
format-opt-article = Artikel & Sachbücher\n\
format-opt-short = Kurzgeschichten\n\
format-opt-screenplay = Drehbücher\n\
format-opt-poetry = Lyrik\n\
appearance-title = Zweisprachige Architektur (29 Sprachen)\n\
appearance-desc = Die App-Sprache steuert Menüs, Navigation und Layout-Richtung (automatisches RTL für Arabisch und Hebräisch). Die Schreibsprache steuert Editoren unabhängig und wird direkt im Konto gespeichert.\n\
offline-packs-title = Dezentrale Offline-Sprachpakete\n\
offline-packs-desc = ExoNovel-Geräteinstallationen laden jedes gewählte Sprachpaket direkt aus dem öffentlichen schreibgeschützten Willow-Namespace von ExoNovel:\n\
save-theme-lang-btn = Sprachpräferenzen speichern\n\
sync-title = Schreiben im Web, auf dem Gerät & Team-Sync\n\
sync-sub = Nutze ExoNovel kostenlos im Browser und kostenlos offline auf deinen Geräten. Die Zusammenarbeit zwischen Schreibenden synchronisiert in beiden Umgebungen kostenlos.\n\
sync-m1-title = Im Web (Kostenlos)\n\
sync-m1-desc = Cloud-Arbeitsbereich in jedem Browser mit Live-Zusammenarbeit zwischen Autorinnen und Gruppen.\n\
sync-m2-title = Auf deinem Gerät (Kostenlos)\n\
sync-m2-desc = Offline-First-Speicher auf Windows, macOS, Linux, Android und iOS — schreibe überall bei voller lokaler Kontrolle über deine Bibliothek.\n\
sync-m3-title = Eigene Geräte synchronisieren (9 $/Monat · 90 $/Jahr)\n\
sync-m3-desc = Ende-zu-Ende-verschlüsselte Synchronisierung, die deine eigenen Offline-Geräteentwürfe dauerhaft mit deinem eigenen Web-Profil abgleicht.\n\
sync-collab-title = Zusammenarbeit mit anderen Schreibenden ist überall kostenlos\n\
sync-collab-desc = Sobald du Mitwirkende hinzufügst, erfolgt die Synchronisation zwischen euch automatisch, um Texte zu teilen und gemeinsam zu bearbeiten — unabhängig davon, ob jemand im Browser oder auf dem Smartphone bzw. Laptop arbeitet. Nur das Synchronisieren deiner eigenen Geräteinhalte mit deinem eigenen Web-Profil ist kostenpflichtig.\n\
sync-zk-title = Zero-Knowledge Kryptografische Privatsphäre\n\
sync-zk-desc = Geräte-synchronisierte Entwürfe und geteilte Datenpakete werden vom Absender Ende-zu-Ende verschlüsselt, bevor sie die Relay-Infrastruktur von ExoNovel berühren.\n\
sync-manage-in-profile = Sync-Tarife im Profil ansehen →\n\
notif-channels-title = Plattformübergreifende Zustellungskanäle\n\
notif-email-prefix = Transaktionaler E-Mail-Dienst (notes@exosystems.net)\n\
notif-email-active = · E-Mail-Versand aktiv\n\
notif-email-unset = · Transaktionale E-Mail-Zugangsdaten nicht konfiguriert\n\
notif-send-test = Test-Benachrichtigung & E-Mail senden\n\
notif-in-app-title = In-App Live-Feed & Glocke\n\
notif-in-app-desc = Sofortige Zustellung über den persistenten Echtzeit-Stream.\n\
notif-web-push-title = Browser-Benachrichtigungen\n\
notif-web-push-desc = Native Systembanner über die Browser Notification API.\n\
notif-native-push-title = Native Push-Geräte (Android UnifiedPush + FCM, iOS/macOS APNs, Windows WNS, Linux)\n\
notif-email-title = Transaktionale E-Mail (Hinweise von ExoNovel)\n\
notif-digest-label = E-Mail-Zusammenfassungshäufigkeit\n\
notif-digest-instant = Sofortige E-Mail\n\
notif-digest-daily = Tägliche Zusammenfassung\n\
notif-digest-off = E-Mail deaktiviert\n\
notif-security-title = Sicherheits- & Anmeldewarnungen\n\
notif-security-desc = Hinweis, wenn sich deine souveräne Identität auf einem neuen Gerät anmeldet.\n\
notif-maturity-title = Reifegrad-Scanner Release-Warnungen\n\
notif-maturity-desc = Benachrichtigung, wenn überwachte Abhängigkeiten neue Versionen veröffentlichen.\n\
notif-save-btn = Kanal-Präferenzen speichern\n\
devices-title = Registrierte Push-Endpunkte (AOTA)\n\
devices-sub = Registriere beliebige Push-Transporte inklusive UnifiedPush (Android/Linux), FCM (Android), APNs (iOS/macOS) oder WNS (Windows).\n\
device-platform-label = Plattform\n\
device-transport-label = Push-Transport\n\
device-label-field = Gerätebezeichnung\n\
device-token-label = Endpunkt-URL oder Geräte-Token\n\
device-token-ph = https://push.example.org/... oder FCM/APNs-Token\n\
device-register-btn = Push-Endpunkt registrieren\n\
device-th-label = Bezeichnung\n\
device-th-platform = Plattform\n\
device-th-transport = Transport\n\
device-th-last-seen = Zuletzt gesehen\n\
device-plat-android = Android\n\
device-plat-ios = iOS\n\
device-plat-linux = Linux\n\
device-plat-macos = macOS\n\
device-plat-windows = Windows\n\
device-plat-web = Webbrowser\n\
device-tr-unifiedpush = UnifiedPush (Offener Standard)\n\
device-tr-fcm = FCM (Firebase Cloud Messaging)\n\
device-tr-apns = APNs (Apple Push Notification service)\n\
device-tr-wns = WNS (Windows Push Notification Services)\n\
device-tr-webpush = Web Push (VAPID)\n\
admin-allowlist-title = Dokumentations-Zugriffsliste\n\
admin-allowlist-sub = Steuert, wer ExoNovels MADR 4.0.0 CMS mit welcher Berechtigung einsehen darf. Klicke unten auf eine did:peer-ID zum Übernehmen.\n\
admin-grant-ph = did:peer:0z... oder E-Mail\n\
admin-note-ph = Notiz\n\
admin-save-access-btn = Zugriff speichern\n\
admin-th-user = Benutzer\n\
admin-th-email = E-Mail\n\
admin-th-status = Status\n\
admin-th-actions = Aktionen\n\
admin-remove-btn = Entfernen\n\
admin-owner-badge = Eigentümer\n\
admin-accounts-title = Registrierte Konten & Rollen\n\
admin-th-role = Rolle\n\
admin-th-did-populate = did:peer (zum Übernehmen klicken)\n\
admin-th-provider = Anbieter\n\
admin-display-name-ph = Anzeigename\n\
admin-pen-name-ph = Künstlername\n\
admin-save-account-btn = Kontoänderungen speichern\n\
preferences-msg-avatar-size = Avatar-Datei darf maximal 2 MB groß sein.\n\
preferences-msg-avatar-processing = Avatar-Varianten werden verarbeitet...\n\
preferences-msg-avatar-saved = Avatar gespeichert.\n\
preferences-msg-saved = Präferenzen gespeichert.\n\
preferences-toast-saved = Präferenzen gespeichert\n\
preferences-toast-notif-saved = Benachrichtigungskanäle aktualisiert\n\
preferences-toast-device-reg = Push-Gerät registriert\n\
preferences-toast-test-sent = Testbenachrichtigung & E-Mail gesendet\n\
preferences-msg-allowlist-updated = Zugriffsliste aktualisiert.\n\
preferences-toast-tier-updated = Zugriffsebene aktualisiert\n\
preferences-toast-entry-removed = Eintrag aus Zugriffsliste entfernt\n\
preferences-msg-account-updated = Konto aktualisiert.\n\
preferences-toast-perm = Browser-Benachrichtigungsberechtigung:\n\
connected-apps-title = Verbundene Apps & Google Docs\n\
connected-apps-desc = Die primäre Anmeldung nutzt nur Identitätsbereiche (openid, email, profile). Verbinde hier optional Google Drive & Google Docs zum Erstellen, Prüfen und Sichern deiner Markdown-Manuskripte und ADRs.\n\
google-drive-docs-title = Google Drive & Google Docs\n\
connect-google-btn = Google Drive & Docs verbinden\n\
reconnect-google-btn = Google neu verbinden\n\
disconnect-revoke-btn = Trennen & Widerrufen\n\
status-connected = Verbunden\n\
status-not-connected = Nicht verbunden\n\
status-revoked = Widerrufen\n\
conn-primary-signin-title = Primäre Anmeldemethoden (nur Identität)\n\
conn-loading-accounts = Verknüpfte Konten werden geladen...\n\
conn-session-auth = Mit Sitzungsdaten authentifiziert.\n\
conn-signin-linked = Anmeldung verknüpft\n\
conn-scopes-prefix = Bereiche:\n\
conn-auth-acct-fallback = Autorisiertes Google-Konto\n\
conn-drive-optional-desc = Optionale dateibezogene Autorisierung (drive.file) zum Erstellen von Google Docs, Speichern von .md-Backups und Durchsuchen von ExoNovel Docs.\n\
conn-revoke-confirm-msg = ExoNovels Google Drive & Docs OAuth-Tokens widerrufen? Dies trennt Drive-Sicherungen, ohne deine primäre ExoNovel-Anmeldung zu beeinträchtigen.\n\
conn-revoke-confirm-btn = Widerruf bestätigen\n\
conn-redirect-configured = OAuth-Weiterleitungs-URI (beim Anbieter konfiguriert)\n\
conn-redirect-missing = Serverseitige Google-OAuth-Zugangsdaten erforderlich\n\
conn-copy-uri-btn = URI kopieren\n\
conn-toast-uri-copied = Weiterleitungs-URI kopiert\n\
drive-ws-title = Google Docs & Drive Arbeitsbereich\n\
drive-ws-sub = Erstelle bearbeitbare Google Docs aus Markdown, durchsuche ExoNovel-Ordner, zeige Vorschauen an oder verwalte den Papierkorb.\n\
drive-new-doc-btn = + Neues Google Doc\n\
drive-new-folder-btn = + Neuer Ordner\n\
drive-viewing-trash = Papierkorb aktiv\n\
drive-trash-btn = Papierkorb\n\
drive-refresh-files = Dateien aktualisieren\n\
drive-ready-suffix = ist in Google Docs bereit.\n\
drive-open-gdocs = In Google Docs öffnen ↗\n\
drive-new-folder-ph = Name des neuen Ordners...\n\
drive-create-folder-btn = Ordner erstellen\n\
drive-doc-title-label = Dokumenttitel\n\
drive-doc-title-ph = Kapitel 1 — Gliederung\n\
drive-target-fmt-label = Zielformat\n\
drive-fmt-gdoc = Google Doc (nativ bearbeitbar)\n\
drive-fmt-md = Markdown (.md)\n\
drive-md-content-label = Markdown-Inhalt\n\
drive-md-content-ph = # Titel — Szene oder Notizen in Markdown schreiben...\n\
drive-save-btn = In Google Docs / Drive speichern\n\
drive-my-drive-root = Meine Ablage\n\
drive-search-ph = Drive-Dateien durchsuchen...\n\
drive-search-btn = Suchen\n\
drive-loading-files = Dateien aus Google Drive werden geladen...\n\
drive-th-name = Name\n\
drive-th-type = Typ\n\
drive-th-modified = Geändert\n\
drive-th-actions = Aktionen\n\
drive-mime-folder = Ordner\n\
drive-mime-gdoc = Google Doc\n\
drive-mime-gsheet = Google Tabelle\n\
drive-mime-gslides = Google Präsentation\n\
drive-mime-md = Markdown (.md)\n\
drive-mime-txt = Text\n\
drive-mime-file = Datei\n\
drive-save-inline = Speichern\n\
drive-open-doc-btn = Doc öffnen ↗\n\
drive-open-btn = Öffnen ↗\n\
drive-preview-btn = Vorschau\n\
drive-rename-btn = Umbenennen\n\
drive-restore-btn = Wiederherstellen\n\
drive-delete-forever-btn = Endgültig löschen\n\
drive-trash-item-btn = In Papierkorb\n\
drive-confirm-perm-delete = Endgültig löschen bestätigen?\n\
drive-empty-files = Keine Dateien in dieser Ansicht. Klicke '+ Neues Google Doc' oder sichere ADRs aus Docs & ADRs.\n\
drive-close-preview = Vorschau schließen\n\
drive-toast-doc-copied = Dokumentinhalt kopiert\n\
conn-msg-list-err = Google Drive & Docs Dateien konnten nicht aufgelistet werden.\n\
conn-msg-load-err = Verbundene Apps konnten nicht geladen werden.\n\
conn-msg-revoking = Google-Tokens werden getrennt und widerrufen...\n\
conn-msg-revoked = Google Drive & Docs getrennt und OAuth-Tokens widerrufen.\n\
conn-toast-revoked = Google Drive & Docs widerrufen\n\
conn-msg-revoke-err = Google-Autorisierung konnte nicht widerrufen werden.\n\
conn-msg-creating-folder = Ordner wird erstellt...\n\
conn-toast-folder-created = Ordner in Google Drive erstellt\n\
conn-msg-folder-err = Ordnererstellung fehlgeschlagen.\n\
conn-msg-doc-required = Bitte sowohl Titel als auch Markdown-Inhalt eingeben.\n\
conn-msg-converting-gdoc = Markdown wird in natives Google Doc umgewandelt...\n\
conn-msg-uploading-md = Markdown-Datei wird zu Google Drive hochgeladen...\n\
conn-toast-doc-saved = Dokument in Google Docs / Drive gespeichert\n\
conn-msg-preview-err = Vorschau fehlgeschlagen.\n\
conn-toast-item-updated = Drive-Eintrag aktualisiert\n\
conn-msg-action-err = Aktion fehlgeschlagen.\n\
tab-overview = Übersicht\n\
tab-works = Werke\n\
tab-groups = Gruppen & Einladungen\n\
tab-wallet = Wallet\n\
tab-paywalls = Paywalls\n\
tab-plans = Tarife\n\
tab-personas = Personas\n\
tab-langs-alerts = Sprachen & Benachrichtigungen\n\
profile-tabs-aria = Profilbereiche\n\
profile-overview-title = Öffentliche Autorenübersicht\n\
profile-overview-desc = Deine souveräne Stammidentität verankert Werke, Personas, Gruppeneinladungen, Paywalls und Leserkäufe.\n\
active-persona-label = Aktive Persona\n\
profile-sync-plan-label = Persönlicher Sync-Tarif:\n\
personas-label = Personas\n\
groups-label = Gruppen\n\
referrals-label = Empfehlungen\n\
profile-referred-by = Eingeladen zu ExoNovel von:\n\
profile-personas-unit = Arbeitsbereichsschlüssel\n\
profile-groups-unit = Kollaborationsgruppen\n\
profile-referrals-unit = beigetretene Autoren\n\
works-title = Verfasste & geforkte Werke\n\
works-desc = Jedes Manuskript speichert kollaborative Loro-CRDT-Änderungen neben Willow-Meadowcap-Pfaden deiner aktiven Persona.\n\
works-novels-title = Romane & Fortsetzungen\n\
works-novels-desc = Kapitel und Szenen mit kostenlosen oder kostenpflichtigen Leserzugängen unter Paywalls konfiguriert.\n\
works-scripts-title = Drehbücher & Lyrik\n\
works-scripts-desc = Mehrsprachiger Texteditor (29 Sprachen + RTL) mit Live-Kollaborationscursorn.\n\
groups-title = Gruppen & Manager\n\
groups-composer-label = Gruppeneditor\n\
groups-name-label = Gruppenname\n\
groups-name-ph = z. B. Anthologie-Redaktion\n\
groups-desc-label = Beschreibung\n\
groups-desc-ph = Woran diese Gruppe gemeinsam arbeitet\n\
groups-create-btn = Gruppe erstellen\n\
groups-invite-btn = In Gruppe einladen\n\
groups-set-member-btn = Als Mitglied setzen\n\
groups-assign-mgr-btn = Zum Manager ernennen\n\
invites-title = Einladungen & Empfehlungen\n\
invites-sub = Autorinnen einladen und beigetretene Empfehlungen verfolgen.\n\
invites-create-btn = Einladungslink erstellen\n\
invites-th-scope = Einladungsbereich\n\
invites-th-link = Einladungslink\n\
invites-th-uses = Nutzungen\n\
invites-th-created = Erstellt\n\
invites-scope-general = Allgemeine ExoNovel-Einladung\n\
wallet-tab-title = Verbundene Wallets & Zahlungswege\n\
wallet-tab-sub = Wähle, wie du Leserzahlungen empfängst und persönliche Geräte-Sync-Tarife über unterstützte Lokalwährungs-, Karten- und Krypto-Wege bezahlst.\n\
wallet-selected-rail-title = Aktiver Abwicklungsweg\n\
wallet-selected-rail-desc = Wird bei der Aktivierung persönlicher Sync-Tarife und für Autoren-Auszahlungen genutzt.\n\
wallet-card-desc = Karten, Apple Pay, Google Pay und lokale Währungsabwicklungen.\n\
wallet-solana-desc = Sekundenschnelle Stablecoin-Auszahlungen und direkte Leser-Freischaltungen.\n\
wallet-btc-desc = Günstige Lightning-Rechnungen für globale Mikro-Freischaltungen und Abos.\n\
ledger-wallet-title = Wallet-Transaktionen & Belege\n\
ledger-wallet-sub = Vollständiges Protokoll aus Autoreneinnahmen, Leser-Quittungen und Sync-Tarifgebühren.\n\
paywalls-title = Autoren-Paywalls & Leserzugang\n\
paywalls-sub = Lege fest, welche Geschichten, Kapitel oder Weltbau-Elemente für alle kostenlos bleiben und welche gegen Zahlung freigeschaltet werden — mit automatischer Aufteilung an Mitwirkende.\n\
paywalls-free-tier-title = Kostenlose öffentliche Kapitel\n\
paywalls-free-tier-desc = Halte Eröffnungskapitel oder ganze Bücher offen, damit neue Lesende dein Werk entdecken können.\n\
paywalls-unlock-title = Paywalls pro Geschichte & Kapitel\n\
paywalls-unlock-desc = Setze direkte Freischaltpreise in deiner bevorzugten Währung für Karten, Mobile Money und Krypto-Wallets.\n\
paywalls-splits-title = Automatische Anteile für Mitwirkende\n\
paywalls-splits-desc = Wenn ihr in einer Gruppe gemeinsam schreibt, werden Paywall-Erlöse sauber auf jede genannte Person aufgeteilt.\n\
paywalls-ledger-title = Leser-Freischaltungen & Paywall-Aktivität\n\
paywalls-ledger-sub = Von dir freigeschaltete Werke sowie bezahlte Freischaltungen deiner eigenen Geschichten.\n\
paywalls-empty = Noch keine Paywall-Verkäufe oder Leser-Freischaltungen verzeichnet.\n\
plans-title = Schreibzugang & Persönliche Geräte-Sync-Tarife\n\
plans-sub = Schreiben im Web ist kostenlos, Schreiben auf deinen eigenen Geräten ist kostenlos, und die Team-Synchronisierung zwischen Schreibenden ist immer kostenlos — selbst wenn eine Person im Web und eine andere offline auf dem Gerät arbeitet. Ein bezahlter Tarif gilt nur für den Abgleich deiner eigenen Geräte-Entwürfe mit deinem eigenen Web-Profil.\n\
plans-payment-rail-note = Gewählter Zahlungsweg\n\
plans-change-wallet-btn = Im Wallet konfigurieren →\n\
plans-free-title = Web, Gerät & Zusammenarbeit (Kostenlos)\n\
plans-free-desc = Kostenloses Schreiben im Web, kostenloses lokales Schreiben auf Geräten und kostenlose Team-Synchronisation.\n\
wallet-peer-prefix = ExoNovel-Dienst-Empfänger-Peer:\n\
wallet-currency-label = Lokale Währung\n\
wallet-rail-label = Standard-Zahlungsweg (AOTA)\n\
wallet-cur-usd = USD (9 $/Monat · 90 $/Jahr)\n\
wallet-cur-eur = EUR (9 €/Monat · 90 €/Jahr)\n\
wallet-cur-gbp = GBP (7 £/Monat · 70 £/Jahr)\n\
wallet-cur-pln = PLN (36 zł/Monat · 360 zł/Jahr)\n\
wallet-cur-ngn = NGN (13.500 ₦/Monat · 135.000 ₦/Jahr)\n\
wallet-cur-kes = KES (1.200 KSh/Monat · 12.000 KSh/Jahr)\n\
wallet-cur-inr = INR (750 ₹/Monat · 7.500 ₹/Jahr)\n\
wallet-cur-brl = BRL (45 R$/Monat · 450 R$/Jahr)\n\
wallet-rail-card = Karte / Lokales Zahlungsgateway\n\
wallet-rail-solana = Solana Pay / USDC\n\
wallet-rail-btc = Bitcoin Lightning (L402)\n\
wallet-rail-eth = Base / Arbitrum / Ethereum\n\
wallet-rail-mobile = Mobile Money / PIX / UPI\n\
wallet-free-current = Aktueller Tarif (Kostenlos)\n\
wallet-free-switch = Zu Kostenlos wechseln\n\
wallet-mo-title = Persönliche Sync Monatlich (9 $/Monat Äquiv.)\n\
wallet-mo-desc = Verschlüsselte Synchronisierung deiner eigenen Offline-Geräteentwürfe mit deinem eigenen Web-Profil.\n\
wallet-mo-active = Aktiv (Monatlich)\n\
wallet-mo-activate = 9 $/Monat aktivieren\n\
wallet-yr-title = Persönliche Sync Jährlich (90 $/Jahr Äquiv.)\n\
wallet-yr-desc = 2 Monate gratis · vollständige verschlüsselte Sync zwischen deinen eigenen Geräten und deinem Web-Profil.\n\
wallet-yr-active = Aktiv (Jährlich)\n\
wallet-yr-activate = 90 $/Jahr aktivieren\n\
ledger-th-type = Typ\n\
ledger-th-amount = Betrag\n\
ledger-th-cycle = Zyklus\n\
ledger-th-details = Details\n\
ledger-th-date = Datum\n\
personas-title = Künstlernamen & Personas\n\
personas-sub = Jede Persona besitzt einen eigenen kryptografischen Subspace-Schlüssel samt Avatar unter deiner Stammidentität.\n\
persona-default-writer = Autor\n\
persona-use-btn = Verwenden\n\
persona-edit-btn = Persona & Avatar bearbeiten\n\
persona-display-ph = Anzeigename\n\
persona-handle-ph = Handle (@kürzel)\n\
persona-bio-ph = Biografie\n\
persona-save-btn = Persona speichern\n\
persona-create-heading = Neue Persona erstellen\n\
persona-composer-label = Persona-Editor\n\
persona-display-label = Anzeigename der Persona\n\
persona-handle-label = Handle\n\
persona-bio-label = Biografie\n\
persona-create-btn = Persona erstellen\n\
langs-quick-title = Persönliche Schnellwechselsprachen\n\
langs-quick-sub = Wähle aus den 29 unterstützten Sprachen diejenigen für die kompakte Kopfleiste und deinen Schreibeditor. Alle 29 bleiben in den Präferenzen verfügbar.\n\
langs-pinned-ui-label = Angeheftete UI-Ausgabesprachen\n\
langs-pinned-input-label = Angeheftete Eingabesprachen\n\
langs-save-btn = Schnellwechsellisten speichern\n\
activity-notif-title = Aktivitäts-, Kollaborations- & Verkaufsbenachrichtigungen\n\
activity-notif-sub = Bestimme, welche Persona-Aktivitäten, Gruppeneinladungen, Paywall-Erlöse und Leserkäufe Benachrichtigungen auslösen.\n\
notif-earn-title = Autoren-Paywall-Einnahmen\n\
notif-earn-desc = Hinweis, wenn ein Leser deine bezahlten Werke freischaltet.\n\
notif-purch-title = Leserkäufe & Belege\n\
notif-purch-desc = Quittungen beim Kauf von Werken oder Paywalls anderer Autoren.\n\
notif-inv-title = Gruppeneinladungen & Rollenwechsel\n\
notif-inv-desc = Einladungen und Manager-Zuweisungen.\n\
notif-ref-title = Empfehlungs-Registrierungen\n\
notif-ref-desc = Hinweise, wenn sich jemand über deinen persönlichen Einladungslink registriert.\n\
notif-collab-title = Co-Autor- & Mitarbeiter-Notizen\n\
notif-collab-desc = Inline-Änderungen und Antworten in gemeinsamen Werken.\n\
notif-fork-title = Erwähnungen & Forks\n\
notif-fork-desc = Warnungen, wenn deine Bausteine mit Urhebernennung geforkt werden.\n\
activity-notif-save-btn = Aktivitätsbenachrichtigungen speichern\n\
profile-toast-persona-created = Persona erstellt\n\
profile-toast-persona-updated = Persona aktualisiert\n\
profile-msg-img-size = Bild darf maximal 2 MB groß sein.\n\
profile-toast-avatar-done = Persona-Avatar verarbeitet\n\
profile-toast-group-created = Gruppe erstellt\n\
profile-toast-role-updated = Mitgliedsrolle aktualisiert\n\
profile-toast-invite-copied = Einladungslink kopiert\n\
profile-toast-plan-updated = Sync-Tarif aktualisiert\n\
profile-toast-langs-saved = Sprach-Schnellwechsler gespeichert\n\
profile-toast-notif-saved = Aktivitätsbenachrichtigungen gespeichert\n\
cms-all = Alle\n\
cms-on-hold = Pausiert\n\
cms-ready = Bereit\n\
cms-accepted = Akzeptiert\n\
cms-proposed = Vorgeschlagen\n\
cms-search-placeholder = ADRs durchsuchen...\n\
cms-sidebar-title = MADR 4.0.0 CMS\n\
cms-records-count-suffix = Datensätze\n\
cms-hide = Ausblenden\n\
cms-show-explorer = Explorer einblenden\n\
cms-read-adr = ADR lesen\n\
cms-new-madr = + Neuer MADR 4.0\n\
cms-tooling-proposals = Tooling & Vorschläge\n\
cms-inference-runtimes = KI-Inferenz-Laufzeiten\n\
cms-owner-tasks = Owner- & Ops-Aufgaben\n\
cms-connect-google-docs = Google Docs verbinden\n\
cms-backup-drive = Archiv auf Drive sichern\n\
cms-backup-title = Alle ExoNovel-Dokumente auf Google Drive sichern\n\
cms-backup-dest-default = Ziel: ExoNovel Docs/\n\
cms-backup-dest-prefix = Ziel: ExoNovel Docs/ auf\n\
cms-backup-fmt-gdoc = Google Docs (bearbeitbar)\n\
cms-backup-fmt-md = Markdown (.md)\n\
cms-backup-start = Backup starten\n\
cms-drive-synced-suffix = ist in Google Drive synchronisiert.\n\
cms-open-drive-btn = In Google Docs / Drive öffnen ↗\n\
cms-select-adr-prompt = Wähle einen Eintrag im linken Explorer aus.\n\
cms-words-suffix = Wörter\n\
cms-min-read-suffix = Min. Lesezeit\n\
cms-trans-canonical = Kanonisch (EN)\n\
cms-trans-translated = Übersetzt\n\
cms-trans-fallback-en = Englischer Fallback\n\
cms-save-gdoc-btn = In Google Docs speichern\n\
cms-save-md-btn = .md auf Drive sichern\n\
cms-edit-adr-btn = Im MADR CMS bearbeiten\n\
cms-deps-title = Zugrundeliegende Tooling-Abhängigkeiten & Blocker\n\
cms-deps-empty = Diesem ADR sind keine blockierenden Tooling-Abhängigkeiten zugeordnet.\n\
cms-deps-installed = installiert\n\
cms-deps-latest = aktuell\n\
cms-deps-req = erf.\n\
cms-deps-constraint-ph = >=0.4.0\n\
cms-deps-cap-ph = Blockierende Funktion...\n\
cms-deps-satisfied = Erfüllt\n\
cms-deps-link-btn = Abhängigkeit verknüpfen\n\
cms-editor-title-edit = MADR 4.0.0 Datensatz bearbeiten\n\
cms-editor-title-new = Neuen MADR 4.0.0 ADR erstellen\n\
cms-editor-context-label = MADR-Editor\n\
cms-field-title = Titel\n\
cms-field-rel-path = Relativer Pfad (z. B. docs/adr/010-short.md)\n\
cms-field-status = Status\n\
cms-field-makers = Entscheidungsträger\n\
cms-field-context = Kontext & Problemstellung\n\
cms-field-drivers = Entscheidungstreiber\n\
cms-field-outcome = Entscheidungsergebnis\n\
cms-field-raw-md = Vollständiges Markdown (optionale Überschreibung)\n\
cms-save-adr-btn = ADR speichern\n\
cms-status-opt-proposed = proposed\n\
cms-status-opt-on-hold = on-hold (wartet auf Tooling)\n\
cms-status-opt-ready = ready-to-build\n\
cms-status-opt-accepted = accepted\n\
cms-status-opt-deprecated = deprecated\n\
cms-status-opt-superseded = superseded\n\
cms-status-opt-rejected = rejected\n\
cms-on-this-page = Auf dieser Seite\n\
cms-tooling-catalog-title = Tooling-Katalog & Release-Scanner\n\
cms-tooling-catalog-sub = Deterministisches crates.io-Versionsmonitoring verknüpft mit Architekturentscheidungen.\n\
cms-run-scan = crates.io-Scan starten\n\
cms-th-tool = Tool\n\
cms-th-installed = Installiert\n\
cms-th-latest = Aktuell\n\
cms-th-purpose = Zweck\n\
cms-add-tool-heading = Weiteres Crate / Tool verfolgen\n\
cms-add-tool-key-ph = Schlüssel (z. B. reqwest)\n\
cms-add-tool-name-ph = Anzeigename\n\
cms-add-tool-pkg-ph = crates.io-Paket\n\
cms-add-tool-ver-ph = Installierte Version\n\
cms-add-tool-purpose-ph = Warum ExoNovel es benötigt\n\
cms-add-tool-btn = Tool hinzufügen\n\
cms-ai-providers-title = KI-Inferenzanbieter & aktive Modelle\n\
cms-ai-providers-sub = Konfigurierte lokale CLI-Laufzeit und verifizierte aktive Modelle dieser Umgebung.\n\
cms-refresh-models = Live-Modelle aktualisieren\n\
cms-ai-empty = Keine aktiven Inferenzanbieter in dieser Umgebung konfiguriert.\n\
cms-model-deprecated = Veraltet\n\
cms-model-cooldown = Abklingzeit\n\
cms-model-active = Aktiv\n\
cms-maturity-proposals-title = Reifegrad-Vorschläge\n\
cms-maturity-proposals-sub = Erzeuge Upgrade-Empfehlungen für verfolgte Abhängigkeiten und akzeptiere oder verwerfe Vorschläge.\n\
cms-generate-proposal = Vorschlag erzeugen\n\
cms-proposals-empty = Keine Vorschläge im gewählten Filter. Wähle oben ein Tool und klicke Vorschlag erzeugen.\n\
cms-proposal-filter-all = Alle\n\
cms-proposal-filter-open = Offen\n\
cms-proposal-filter-accepted = Akzeptiert\n\
cms-proposal-filter-dismissed = Verworfen\n\
cms-proposal-status-prefix = Status:\n\
cms-proposal-accept = Vorschlag akzeptieren\n\
cms-proposal-dismiss = Verwerfen\n\
cms-proposal-reopen = Wieder öffnen\n\
cms-proposal-rationale = Diagnose / Begründung:\n\
cms-owner-tasks-title = Externe Owner- & Ops-Checkliste\n\
cms-owner-tasks-sub = Aufgaben mit externen Konsolen oder Produktionsserver-Konfiguration. Nach Erledigung abhaken, um den Status in der Datenbank zu speichern.\n\
cms-task-done = Erledigt\n\
cms-task-pending = Offen\n\
cms-msg-redirecting-google = Weiterleitung zur Google-Zustimmung...\n\
cms-msg-saving-gdoc = Aktiver ADR wird in Google Docs gespeichert...\n\
cms-msg-saving-md = Aktiver ADR wird als .md auf Google Drive gesichert...\n\
cms-msg-save-failed = Speichern fehlgeschlagen.\n\
cms-toast-saved-drive = In Google Docs / Drive gespeichert\n\
cms-msg-backing-up-archive = Vollständiges ExoNovel Docs & ADR Archiv wird auf Google Drive gesichert (kann kurz dauern)...\n\
cms-msg-archive-failed = Archiv-Backup fehlgeschlagen.\n\
cms-toast-archive-backed-up = Archiv auf Google Drive gesichert\n\
cms-toast-task-done = Aufgabe als erledigt markiert\n\
cms-toast-task-reopened = Aufgabe wieder geöffnet\n\
cms-toast-task-failed = Aufgabe konnte nicht aktualisiert werden\n\
cms-msg-saving-adr = MADR 4.0.0 Datensatz wird gespeichert...\n\
cms-msg-missing-token = Sitzungs-Token fehlt.\n\
cms-toast-adr-saved = ADR im CMS gespeichert\n\
cms-toast-dep-linked = Tooling-Abhängigkeit mit ADR verknüpft\n\
cms-msg-scanning-crates = Deterministischer crates.io-Release-Scan läuft...\n\
cms-toast-scan-done = crates.io-Release-Katalog aktualisiert\n\
cms-msg-scan-failed = Scan fehlgeschlagen.\n\
cms-msg-refreshing-models = Live-Modelle über konfigurierte Inferenzanbieter werden aktualisiert...\n\
cms-toast-models-refreshed = Live-Anbietermodelle aktualisiert\n\
cms-msg-generating-proposal = Aktive Inferenz-Analyse für Vorschlag läuft...\n\
cms-toast-proposal-generated = Reifegrad-Vorschlag erstellt\n\
cms-toast-proposal-updated = Vorschlagsstatus aktualisiert\n\
cms-toast-tool-added = Tool zum Reifegrad-Katalog hinzugefügt\n\
cms-toast-copied = Text in Zwischenablage kopiert\n\
nav-group-account = Konto\n\
nav-group-workspace = Arbeitsbereich & Zustellung\n\
nav-group-admin = Verwaltung\n\
nav-group-writing = Schreiben & Zusammenarbeit\n\
nav-group-commerce = Abrechnung & Lesezugang\n\
nav-group-personalization = Personalisierung\n\
settings-resize-aria = Navigationsleiste in der Breite anpassen\n",
        _ => "\
nav-sign-in = Sign in\n\
nav-toggle-menu = Toggle navigation menu\n\
nav-privacy = Privacy Policy\n\
nav-terms = Terms of Service\n\
live-connection-title = Global Real-Time Stream Connection\n\
ui-lang-aria = App Output Language\n\
input-lang-aria = Writing Input Language\n\
notifications-aria = Notifications\n\
switch-persona = Switch Persona\n\
create-personas-prompt = Create Personas in your Profile\n\
default-persona-label = Default Persona\n\
active-badge = Active\n\
mark-all-read = Mark all read\n\
mark-read = Mark read\n\
no-notifications = No notifications yet.\n\
refresh-label = Refresh\n\
cancel-label = Cancel\n\
dismiss-label = Dismiss\n\
save-changes-short = Save\n\
confirm-label = Confirm\n\
copy-text-label = Copy Text\n\
pen-name-label = Pen name:\n\
pending-did-label = Pending did:peer\n\
workspace-desc = Your writing desk keeps everything you're working on close at hand—whether you're outlining your very first story for class, shaping characters with friends, or drafting poems on your own. Every character, timeline, scene, and comment you add stays credited to your pen name, so you can share ideas freely, learn together, and know your work always remains yours.\n\
btn-profile-billing = Profile, Personas & Billing\n\
btn-connected-apps = Connected Apps & Google Docs\n\
btn-open-cms = Open MADR Docs CMS\n\
workspace-key-label = Active Persona Workspace Key (did:peer)\n\
choose-image-btn = Choose Profile Photo\n\
persona-choose-avatar-btn = Choose Persona Photo\n\
desk-start-eyebrow = Start a New Project\n\
desk-start-title = What would you like to write today?\n\
desk-starter-novel-hint = Ready on your shelf: Story Outline · Character Cards · Chapter 1 Draft · World Timeline.\n\
desk-starter-article-hint = Ready on your shelf: Thesis Hook · Section Outline · Source Notes · Final Essay Draft.\n\
desk-starter-short-hint = Ready on your shelf: Opening Hook · Turning Point · Character Beat · Scene Draft.\n\
desk-starter-script-hint = Ready on your shelf: Scene Heading · Cast & Voices · Beat Sheet · Script Pages.\n\
desk-starter-poetry-hint = Ready on your shelf: Stanza Draft · Line Rhythm Notes · Collection Order · Author Dedication.\n\
desk-blocks-eyebrow = Story Building Blocks\n\
desk-blocks-title = Your Shelf of Story Pieces\n\
desk-block-chars-title = Characters\n\
desk-block-chars-desc = Keep track of heroes, rivals, sidekicks, and their backstories—even bring a friend's character into a scene with clear credit.\n\
desk-block-plots-title = Plots\n\
desk-block-plots-desc = Sketch turning points, chapter beats, and surprises so your story moves smoothly from beginning to ending.\n\
desk-block-timelines-title = Timelines\n\
desk-block-timelines-desc = Line up what happened first, next, and years before—helpful when writing mysteries, trilogies, or class projects together.\n\
desk-block-themes-title = Themes\n\
desk-block-themes-desc = Pin the big questions, moods, and motifs at the heart of your writing so every co-writer stays on the same page.\n\
desk-scratchpad-eyebrow = Scratchpad\n\
desk-scratchpad-label = Quick Idea Scratchpad\n\
desk-scratchpad-title = Jot Down a Quick Idea\n\
desk-note-title-label = Working title or prompt\n\
desk-note-title-ph = e.g. Rain over the library steps...\n\
desk-note-body-label = Opening lines or scene sketch\n\
desk-note-body-ph = Write a few sentences, test dialogue, or switch your writing language above...\n\
desk-note-keep-btn = Pin Idea on Desk\n\
desk-note-clear-btn = Clear\n\
desk-note-pinned-msg = Idea pinned on your desk for this session\n\
guides-eyebrow = Dear World · User Guides\n\
guides-sidebar-title = Getting Started\n\
guides-sidebar-sub = Step-by-step guides written for High School and University writers exploring collaborative storytelling on ExoNovel.\n\
guides-open-desk-btn = Go to Your Desk\n\
guides-signin-btn = Sign in to Start Writing\n\
guides-loading = Opening your Dear World guide...\n\
guides-empty = Select a guide chapter from the list to begin.\n\
toast-did-copied = did:peer ID copied\n\
toast-switched-persona = Switched persona\n\
oauth-connected-google = Google Drive & Docs connected successfully.\n\
theme-system-label = System theme\n\
theme-light-label = Light theme\n\
theme-dark-label = Dark theme\n\
role-admin = Admin\n\
role-moderator = Moderator\n\
role-user = User\n\
status-reader = Reader\n\
status-pending = Pending\n\
status-suspended = Suspended\n\
status-restricted = Restricted\n\
status-manager = Manager\n\
status-member = Member\n\
landing-eyebrow = ExoNovel · Web, Desktop & Mobile\n\
landing-title-row1 = Collaborative Creative Writing\n\
landing-title-row2 = Where Authors Own Their Work\n\
landing-title-row3 = ... by Design!\n\
landing-lead = ExoNovel is a collaborative creative writing and local-first publishing platform built for authors, co-writers, and literary teams. Draft novels, essays, short stories, screenplays, and poetry together while keeping cryptographic ownership of your manuscripts.\n\
landing-cta = Sign in to ExoNovel\n\
landing-cap-aria = Platform capabilities\n\
landing-prev-slide = Previous slide\n\
landing-next-slide = Next slide\n\
slide-1-tab = Your Writing, Your Rights\n\
slide-1-title = Create Together, But Remain the Owner of Each of Your Ideas\n\
slide-1-p1 = ExoNovel is a creative writing space where you can build stories on your own or team up with friends, co-writers, and writing groups in real time. Instead of losing track of who wrote what inside messy shared documents, every character profile, plot outline, timeline, theme, and scene remains clearly and securely credited to its creator, including the comments you make on the creations of others.\n\
slide-1-p2 = That means collaborating never costs you your work. You decide what remains private, what your group can edit together, and what other writers can borrow and remix into their own stories—with your entirely independent decentralized identity permanently attached to what you made.\n\
slide-2-tab = One Producer, Many Personas\n\
slide-2-title = One Identity, as Many Personas as You Want\n\
slide-2-p1 = Writing a sci-fi story with friends, sharing poetry under a pen name, and drafting nonfiction under your real name shouldn't force you to juggle three separate accounts. ExoNovel supports creating many Personas—each with its own display name/handle, bio, and avatar, operating also as a content proxy (meaning that you can publish as your Persona), and with its own access to your community of readers and collaborators, so that you can even chat as your Persona about your own work and the work of others. ExoNovel creators need not disclose their true identity to the world, but may do so if they wish—all under one entirely independent login.\n\
slide-2-p2 = Switch personas instantly to switch to another Persona workspace, with its own projects, group chats, or genres. Collaborators and readers see the exact Persona display name you chose for each space, while everything you write across all of your Persona spaces remains safely organized and permanently legitimately yours, thanks to your Identity's securely decentralized P2P ID.\n\
slide-3-tab = Write, Wherever You Are\n\
slide-3-title = Online and Offline, Sync All Your Devices\n\
slide-3-p1 = Create right in your browser whenever you're online, or continue writing without the web using our free Linux, macOS, Windows, Android, and iOS apps. Using our free apps can keep you in sync with your work from the web, and for a small monthly fee (or even a smaller annual fee), your online and local creations are always within reach, together in your one virtual writing home.\n\
slide-3-p2 = Start a scene on your phone on the go, polish a chapter on your laptop later, and manually sync your work, on demand, between offline devices and the web whenever you want—or activate our automatic sync so that every draft stays up to date across all your screens.\n\
slide-4-tab = Connect Your Way\n\
slide-4-title = Connect Your Tools, Keep Total Freedom\n\
slide-4-p1 = Your writing shouldn't be trapped inside a single platform. Sign in the way you prefer, write in the formats that fit your project, and link external cloud storage, tools, and workflows for backups, quick exports, publishing, cross-platform projects, or whatever else your creative process needs.\n\
slide-4-p2 = Everything in ExoNovel works together without locking you in. Add connections whenever they help, combine tools as your projects grow, and disconnect any service anytime while enjoying full access to your account, personas, and creations.\n\
landing-modes-title = On the Web, On Your Device, and Synced Together\n\
landing-modes-sub = Write for free in your web browser and in our native device apps. Collaborating with other writers syncs automatically across web and device for free—while keeping your own personal web and device shelves synced automatically is a small monthly or annual fee.\n\
mode-free-badge = Free Forever\n\
mode-free-title = On the Web & On Your Device\n\
mode-free-desc = Create, invite co-writers, and collaborate across web browsers and our free Linux, macOS, Windows, Android, and iOS apps—with free automatic collaboration sync between co-writers on web and device.\n\
mode-monthly-badge = Personal Sync\n\
mode-monthly-title = Personal Sync — $9 / month\n\
mode-monthly-desc = Keep your own personal drafts on the web and on your offline devices automatically synced with end-to-end encryption ($9/month local currency equivalent).\n\
mode-annual-badge = Annual Personal Sync\n\
mode-annual-title = Personal Sync — $90 / year\n\
mode-annual-desc = Full automatic sync between your own web profile and your personal devices ($7.50/month effective rate in local currency equivalent—two months free compared to monthly).\n\
landing-formats-title = Pick Your Format. Start Creating.\n\
landing-formats-sub = Whether you're writing a novel for fun, drafting a class essay, or scripting a short film, start with a clean layout built for the way you create.\n\
format-novel-title = Novels\n\
format-novel-desc = Build entire worlds from scratch. Organize characters, plot twists, and chapters in one place so big stories never get messy.\n\
format-article-title = Articles & Nonfiction\n\
format-article-desc = Turn ideas and research into sharp essays, blog posts, or school papers with clear outlines and neat sources.\n\
format-short-title = Short Stories\n\
format-short-desc = Nail the hook, the twist, and the ending—great for flash fiction, standalone stories, or your own story collection.\n\
format-screenplay-title = Screenplays & Scripts\n\
format-screenplay-desc = Write scenes and dialogue that look like real movie or video scripts, formatted cleanly as you type.\n\
format-poetry-title = Poetry\n\
format-poetry-desc = Shape rhythm, stanzas, and line breaks that stay right where you put them—from a single poem to a full book.\n\
signin-back-home = ← Back\n\
signin-title = Sign in to ExoNovel\n\
signin-subtitle = Authenticate to resolve your authoritative did:peer identity and open your workspace.\n\
signin-checking-session = Checking existing session...\n\
signin-invite-badge = Personal Invitation\n\
signin-invited-group = invited you to join this group on ExoNovel:\n\
signin-invited-platform = invited you to join ExoNovel.\n\
signin-create-heading = Create account\n\
signin-continue-heading = Continue to ExoNovel\n\
signin-or-email = or email\n\
signin-name-label = Name\n\
signin-email-label = Email\n\
signin-password-label = Password\n\
signin-authenticating = Authenticating...\n\
signin-create-btn = Create account\n\
signin-submit-btn = Sign in\n\
signin-have-account = Already have an account? Sign in\n\
signin-need-account = Need an account? Create one\n\
signin-auth-failed = Authentication failed.\n\
footer-brand-prefix = ExoNovel, from\n\
privacy-page-title = Privacy Policy — ExoNovel\n\
privacy-effective = Effective Date & Last Updated: November 24, 2026\n\
privacy-heading = ExoNovel Privacy Policy\n\
privacy-intro = ExoNovel (https://exonovel.exosystems.net) is a collaborative creative writing and local-first publishing platform operated by Exosystems (https://exosystems.net). This Privacy Policy describes how Exosystems collects, uses, stores, protects, and deletes personal data and Google user data when you use ExoNovel across the web application and companion client builds.\n\
privacy-s1-title = 1. Information We Collect\n\
privacy-s1-intro = We collect only the information needed to authenticate your account, operate your collaborative writing workspace, and deliver features you explicitly enable:\n\
privacy-s1-google-label = Google Sign-In Account Data (openid, email, profile): \n\
privacy-s1-google-body = When you sign in with Google, we receive your verified email address, email verification status, display name, given and family name, profile picture URL, locale, and Google provider account identifier.\n\
privacy-s1-drive-label = Optional Connected Google Drive Data: \n\
privacy-s1-drive-body = Primary sign-in never requests Google Drive permissions. Only if you explicitly connect Google Drive inside ExoNovel's Workspace or Preferences does ExoNovel request Google Drive authorization to list folders, create or rename backup folders, upload your exported Markdown documentation/manuscript archives or session transcripts, and restore or trash files you select inside the Drive Workspace.\n\
privacy-s1-profile-label = Other Sign-In & Author Profile Information: \n\
privacy-s1-profile-body = If you sign in with GitHub or email/password, or customize your workspace, we store your display name, pen name/handle, biography, website, location, preferred publication format, output and input language preferences, timezone, notification preferences, uploaded avatar images (processed server-side into 64px, 256px, and 1024px WebP/JPEG variants), and your cryptographic public key / did:peer identity.\n\
privacy-s1-works-label = Manuscripts & Collaboration Records: \n\
privacy-s1-works-body = Creative works, story bibles, personas, collaboration group memberships, referrals, and invitations that you choose to create or share inside ExoNovel.\n\
privacy-s1-sec-label = Security & Technical Diagnostics: \n\
privacy-s1-sec-body = Sign-in timestamps, IP address, browser User-Agent string, and application health/error logs used strictly to protect accounts against unauthorized access and keep the service reliable.\n\
privacy-s2-title = 2. Cookies & Browser Storage\n\
privacy-s2-body = ExoNovel uses strictly necessary HttpOnly authentication session cookies to maintain your signed-in session, and temporary browser session storage strictly to preserve pending collaboration invite codes across sign-in redirects. ExoNovel does not use third-party advertising cookies, behavioral tracking pixels, or cross-site ad networks.\n\
privacy-s3-title = 3. How We Use Your Information\n\
privacy-s3-li1 = Authenticate your session and resolve your authoritative did:peer identity across web, desktop, and mobile.\n\
privacy-s3-li2 = Display your chosen author persona, pen name, avatar, and language preferences (including right-to-left layout preferences).\n\
privacy-s3-li3 = Operate collaborative story groups, contributor attribution, and group/referral invitations you initiate.\n\
privacy-s3-li4 = Execute user-initiated backups or exports to your connected Google Drive workspace when you explicitly trigger them.\n\
privacy-s3-li5 = Deliver transactional, security, and notification emails you enable (sent from notes@exosystems.net, noreply@exosystems.net, or support@exosystems.net).\n\
privacy-s3-li6 = Prevent fraud, abuse, unauthorized access, and violations of our Terms of Service.\n\
privacy-s4-title = 4. Google API Services User Data Policy (Limited Use) & AI / Content Restrictions\n\
privacy-s4-intro = ExoNovel's use and transfer to any other app of information received from Google APIs will adhere to the Google API Services User Data Policy, including the Limited Use requirements.\n\
privacy-s4-li1-label = No Advertising or Data Brokerage: \n\
privacy-s4-li1-body = We never sell, rent, or transfer Google user data or your writing content to data brokers, advertisers, or third-party marketing platforms.\n\
privacy-s4-li2-label = No Generalized AI Model Training: \n\
privacy-s4-li2-body = ExoNovel does not use Google user data, Google Drive files, or your private manuscripts to train generalized artificial intelligence or machine learning models.\n\
privacy-s4-li3-label = No Image Generation & Zero Tolerance for AI NCII: \n\
privacy-s4-li3-body = ExoNovel is a text-based writing and publishing platform. It does not generate images, video, or audio, and strictly prohibits using ExoNovel or any connected Google APIs for AI-generated Non-Consensual Intimate Imagery (AI NCII).\n\
privacy-s4-li4-label = Human Access Restrictions: \n\
privacy-s4-li4-body = Exosystems personnel do not read your private Google Drive files or private manuscripts unless (a) you explicitly grant consent for support debugging, (b) it is necessary for security or abuse investigation, or (c) it is required to comply with applicable law.\n\
privacy-s5-title = 5. Data Storage, Security & Cryptographic Protection\n\
privacy-s5-body = All traffic to ExoNovel is encrypted in transit using HTTPS/TLS. Hosted web records and account state are stored in managed PostgreSQL databases and managed authentication services on enterprise cloud infrastructure, alongside transactional email delivery. When optional Connected External App offline tokens are stored server-side, they are envelope-encrypted via dedicated key management encryption and isolated from primary login credentials. On device-native installations using peer-to-peer Willow and iroh relay synchronization, payloads are end-to-end encrypted on your device so relay servers remain cryptographically blind to relayed content.\n\
privacy-s6-title = 6. How to Revoke Access, Export & Delete Your Data\n\
privacy-s6-li1-label = Disconnect & Revoke Google Access Anytime: \n\
privacy-s6-li1-body = You can disconnect Google Drive or other optional Connected Apps inside ExoNovel Preferences -> Connected Apps (which revokes active tokens directly through https://oauth2.googleapis.com/revoke without closing your primary ExoNovel account), or revoke access anytime in your Google Account Permissions at https://myaccount.google.com/permissions.\n\
privacy-s6-li2-label = Export & Account Deletion Requests: \n\
privacy-s6-li2-body = You may request a complete export or permanent deletion of your ExoNovel account, Google OAuth profile metadata, personas, avatar assets, and hosted workspace records by emailing support@exosystems.net or exonomyapp@gmail.com. Verified deletion requests are completed within 30 days and remove active personal data from our hosted database and OAuth token store.\n\
privacy-s7-title = 7. Data Retention, Children's Privacy & Policy Updates\n\
privacy-s7-body = We retain personal account data only as long as your account remains active or as necessary to fulfill security, billing, or legal obligations. ExoNovel is not directed to children under 13 (or under 16 where required by applicable regional data protection law), and we do not knowingly collect personal data from minors below the applicable age threshold. If we make material changes to this Privacy Policy, we will update the Effective Date above and notify active account holders where appropriate.\n\
privacy-s8-title = 8. Contact Information\n\
privacy-s8-body = For privacy questions, Google user data inquiries, or deletion requests, contact Exosystems Support at support@exosystems.net or exonomyapp@gmail.com.\n\
terms-page-title = Terms of Service — ExoNovel\n\
terms-effective = Effective Date & Last Updated: November 24, 2026\n\
terms-heading = ExoNovel Terms of Service\n\
terms-intro = By accessing or using ExoNovel (https://exonovel.exosystems.net), operated by Exosystems (https://exosystems.net), you agree to be bound by these Terms of Service and our Privacy Policy (/privacy).\n\
terms-s1-title = 1. Ownership of Your Creative Works\n\
terms-s1-body = You retain full ownership and copyright of all manuscripts, stories, characters, worldbuilding notes, scripts, poetry, annotations, and creative assets you write or publish on ExoNovel. You grant ExoNovel only the limited technical license necessary to store, render, synchronize, back up (when directed by you), and transmit your content according to your workspace and collaboration preferences.\n\
terms-s2-title = 2. Accounts, Keys & Acceptable Use\n\
terms-s2-body = You are responsible for safeguarding your login credentials and sovereign did:peer device keys. You must be at least 13 years old (or the minimum legal age in your jurisdiction) to create an account. You agree not to interfere with platform security, attempt unauthorized access to other users' workspaces, distribute malware, or send automated spam.\n\
terms-s3-title = 3. Prohibited Content & Google APIs Terms of Service Compliance\n\
terms-s3-intro = ExoNovel is a text-focused collaborative creative writing platform and does not provide image, video, or audio generation tools. ExoNovel strictly complies with the Google APIs Terms of Service and enforces zero tolerance for harmful or unlawful content:\n\
terms-s3-li1-label = Strict Prohibition on AI NCII: \n\
terms-s3-li1-body = You may not use ExoNovel or any connected Google APIs (including Google Sign-In or Google Drive integrations) to create, host, upload, synchronize, distribute, or facilitate AI-generated Non-Consensual Intimate Imagery (AI NCII), non-consensual sexualized depictions, or deepfakes.\n\
terms-s3-li2-label = Unlawful & Abusive Material: \n\
terms-s3-li2-body = Child sexual abuse material (CSAM), non-consensual intimate imagery, harassment, hate incitement, and copyright infringement are strictly prohibited and result in immediate account suspension and termination.\n\
terms-s4-title = 4. Connected External Apps (Google Drive & Third-Party Services)\n\
terms-s4-body = Optional integrations such as Google Drive backup are activated only upon your explicit authorization and may be disconnected or revoked at any time in ExoNovel Preferences -> Connected Apps or via your third-party provider's account security page. Your use of third-party services remains subject to their respective terms.\n\
terms-s5-title = 5. Service Availability, Sync Plans & Contact\n\
terms-s5-body = ExoNovel provides hosted web collaboration, local-first device storage, and optional personal device sync plans. Features are provided on an 'as available' basis as the platform evolves. For legal, safety, billing, or support inquiries, contact support@exosystems.net or exonomyapp@gmail.com.\n\
preferences-subtitle = Account, language, sync overview, and notification delivery preferences (purchases, paywalls, and plans live in Profile).\n\
preferences-nav-aria = Preferences sections\n\
sec-account = Account & Sovereign Identity\n\
sec-connected-apps = Connected Apps & Google Docs\n\
sec-appearance-langs = Languages (29)\n\
sec-sync-storage = Web, Device & Collaboration Sync\n\
sec-notifications = Notification Channels\n\
sec-admin-allowlist = Docs Allowlist & Accounts\n\
account-title = Account & Sovereign Identity\n\
save-account-btn = Save Account Preferences\n\
upload-avatar-label = Upload Profile Photo (64px / 256px / 1024px WebP)\n\
root-identity-label = Root Identity (did:peer)\n\
display-name-label = Display Name\n\
pen-name-field-label = Default Pen Name\n\
website-label = Website\n\
location-label = Location & Timezone\n\
preferred-format-label = Preferred Writing Format\n\
timezone-label = Timezone\n\
author-bio-label = Author Bio\n\
format-opt-novel = Novels\n\
format-opt-article = Articles & Nonfiction\n\
format-opt-short = Short Stories\n\
format-opt-screenplay = Screenplays\n\
format-opt-poetry = Poetry\n\
appearance-title = Dual Language Architecture (29 Languages)\n\
appearance-desc = App Language controls menus, navigation, and layout direction (automatic right-to-left for Arabic and Hebrew). Writing Input controls composers independently and saves directly to your account.\n\
offline-packs-title = Decentralized Offline Language Packs\n\
offline-packs-desc = ExoNovel device installs download any selected language pack directly from ExoNovel's public read-only Willow namespace:\n\
save-theme-lang-btn = Save Language Preferences\n\
sync-title = Writing on Web, on Device & Collaboration Sync\n\
sync-sub = Use ExoNovel for free in your browser and free offline on your devices. Team collaboration sync between writers is free across both.\n\
sync-m1-title = On the Web (Free)\n\
sync-m1-desc = Cloud workspace accessible from any browser with live collaboration across writers and groups.\n\
sync-m2-title = On Your Device (Free)\n\
sync-m2-desc = Offline-first storage on Windows, macOS, Linux, Android, and iOS—write anywhere and keep full local control of your library.\n\
sync-m3-title = Sync Your Own Devices ($9/mo · $90/yr)\n\
sync-m3-desc = End-to-end encrypted sync keeping your own offline device drafts continuously aligned with your own web profile.\n\
sync-collab-title = Collaborating With Other Writers Is Free Everywhere\n\
sync-collab-desc = Whenever you add collaborators, sync between collaborators happens automatically so you can share your work and collaborate—no matter whether one writer is in a browser and another is on a desktop or phone app. Only syncing your own device content with your own web profile carries a fee.\n\
sync-zk-title = Zero-Knowledge Cryptographic Privacy\n\
sync-zk-desc = Device-synced drafts and shared payloads are end-to-end encrypted by the sender before passing through ExoNovel's relay infrastructure.\n\
sync-manage-in-profile = View Sync Plans in Profile →\n\
notif-channels-title = Multi-Platform Delivery Channels\n\
notif-email-prefix = Transactional Email Service (notes@exosystems.net)\n\
notif-email-active = · Email Delivery Active\n\
notif-email-unset = · Transactional email credentials not configured\n\
notif-send-test = Send Test Notification & Email\n\
notif-in-app-title = In-App Live Feed & Bell\n\
notif-in-app-desc = Instant delivery over the persistent real-time stream.\n\
notif-web-push-title = Web Browser Notifications\n\
notif-web-push-desc = Native OS banner via browser Notification API.\n\
notif-native-push-title = Native Push Devices (Android UnifiedPush + FCM, iOS/macOS APNs, Windows WNS, Linux)\n\
notif-email-title = Transactional Email (Notes from ExoNovel)\n\
notif-digest-label = Email Digest Frequency\n\
notif-digest-instant = Instant Email\n\
notif-digest-daily = Daily Digest\n\
notif-digest-off = Email Disabled\n\
notif-security-title = Security & Sign-In Alerts\n\
notif-security-desc = Alert when your sovereign identity authenticates on a new device.\n\
notif-maturity-title = Maturity Scanner Release Alerts\n\
notif-maturity-desc = Notify when tracked dependencies ship new versions.\n\
notif-save-btn = Save Channel Preferences\n\
devices-title = Registered Device Push Endpoints (AOTA)\n\
devices-sub = Register any native or web push transport including UnifiedPush (de-Googled Android/Linux), FCM (Google Play Android), APNs (iOS/macOS), or WNS (Windows).\n\
device-platform-label = Platform\n\
device-transport-label = Push Transport\n\
device-label-field = Device Label\n\
device-token-label = Endpoint URL or Device Token\n\
device-token-ph = https://push.example.org/... or FCM/APNs token\n\
device-register-btn = Register Push Endpoint\n\
device-th-label = Label\n\
device-th-platform = Platform\n\
device-th-transport = Transport\n\
device-th-last-seen = Last Seen\n\
device-plat-android = Android\n\
device-plat-ios = iOS\n\
device-plat-linux = Linux\n\
device-plat-macos = macOS\n\
device-plat-windows = Windows\n\
device-plat-web = Web Browser\n\
device-tr-unifiedpush = UnifiedPush (Open Standard)\n\
device-tr-fcm = FCM (Firebase Cloud Messaging)\n\
device-tr-apns = APNs (Apple Push Notification service)\n\
device-tr-wns = WNS (Windows Push Notification Services)\n\
device-tr-webpush = Web Push (VAPID)\n\
admin-allowlist-title = Documentation Access Allowlist\n\
admin-allowlist-sub = Controls who can view ExoNovel's MADR 4.0.0 CMS and at which tier. Click any did:peer below to populate.\n\
admin-grant-ph = did:peer:0z... or email\n\
admin-note-ph = Note\n\
admin-save-access-btn = Save Access\n\
admin-th-user = User\n\
admin-th-email = Email\n\
admin-th-status = Status\n\
admin-th-actions = Actions\n\
admin-remove-btn = Remove\n\
admin-owner-badge = Owner\n\
admin-accounts-title = Registered Accounts & Roles\n\
admin-th-role = Role\n\
admin-th-did-populate = did:peer (click to populate)\n\
admin-th-provider = Provider\n\
admin-display-name-ph = Display Name\n\
admin-pen-name-ph = Pen Name\n\
admin-save-account-btn = Save Account Changes\n\
preferences-msg-avatar-size = Avatar file must be 2 MB or smaller.\n\
preferences-msg-avatar-processing = Processing avatar derivatives...\n\
preferences-msg-avatar-saved = Avatar saved.\n\
preferences-msg-saved = Preferences saved.\n\
preferences-toast-saved = Preferences saved\n\
preferences-toast-notif-saved = Notification channels updated\n\
preferences-toast-device-reg = Push device registered\n\
preferences-toast-test-sent = Test notification & email dispatched\n\
preferences-msg-allowlist-updated = Allowlist updated.\n\
preferences-toast-tier-updated = Allowlist tier updated\n\
preferences-toast-entry-removed = Allowlist entry removed\n\
preferences-msg-account-updated = Account updated.\n\
preferences-toast-perm = Browser notification permission:\n\
connected-apps-title = Connected Apps & Google Docs\n\
connected-apps-desc = Primary sign-in uses minimal identity scopes only (openid, email, profile). Connect Google Drive & Google Docs separately here to create, preview, and back up your Markdown manuscripts and ADRs into Google Docs or .md files.\n\
google-drive-docs-title = Google Drive & Google Docs\n\
connect-google-btn = Connect Google Drive & Docs\n\
reconnect-google-btn = Reconnect Google\n\
disconnect-revoke-btn = Disconnect & Revoke\n\
status-connected = Connected\n\
status-not-connected = Not Connected\n\
status-revoked = Revoked\n\
conn-primary-signin-title = Primary Sign-In Methods (Identity Only)\n\
conn-loading-accounts = Loading linked accounts...\n\
conn-session-auth = Authenticated with session credentials.\n\
conn-signin-linked = Sign-in Linked\n\
conn-scopes-prefix = Scopes:\n\
conn-auth-acct-fallback = Authorized Google account\n\
conn-drive-optional-desc = Optional per-file authorization (drive.file) to create Google Docs, save .md backups, and browse ExoNovel Docs.\n\
conn-revoke-confirm-msg = Revoke ExoNovel's Google Drive & Docs OAuth tokens? This disconnects Drive backups without affecting your main ExoNovel sign-in.\n\
conn-revoke-confirm-btn = Confirm Revoke\n\
conn-redirect-configured = OAuth Redirect URI (Configured with Provider)\n\
conn-redirect-missing = Server Google OAuth Credentials Required\n\
conn-copy-uri-btn = Copy URI\n\
conn-toast-uri-copied = Redirect URI copied\n\
drive-ws-title = Google Docs & Drive Workspace\n\
drive-ws-sub = Create native editable Google Docs from Markdown, browse ExoNovel folders, preview docs as Markdown, or manage Trash.\n\
drive-new-doc-btn = + New Google Doc\n\
drive-new-folder-btn = + New Folder\n\
drive-viewing-trash = Viewing Trash\n\
drive-trash-btn = Trash\n\
drive-refresh-files = Refresh Files\n\
drive-ready-suffix = is ready in Google Docs.\n\
drive-open-gdocs = Open in Google Docs ↗\n\
drive-new-folder-ph = New folder name...\n\
drive-create-folder-btn = Create Folder\n\
drive-doc-title-label = Document Title\n\
drive-doc-title-ph = Chapter 1 — Outline\n\
drive-target-fmt-label = Target Format\n\
drive-fmt-gdoc = Google Doc (Native Editable)\n\
drive-fmt-md = Markdown (.md)\n\
drive-md-content-label = Markdown Content\n\
drive-md-content-ph = # Title — Write your scene or notes in Markdown...\n\
drive-save-btn = Save to Google Docs / Drive\n\
drive-my-drive-root = My Drive\n\
drive-search-ph = Search Drive files...\n\
drive-search-btn = Search\n\
drive-loading-files = Loading files from Google Drive...\n\
drive-th-name = Name\n\
drive-th-type = Type\n\
drive-th-modified = Modified\n\
drive-th-actions = Actions\n\
drive-mime-folder = Folder\n\
drive-mime-gdoc = Google Doc\n\
drive-mime-gsheet = Google Sheet\n\
drive-mime-gslides = Google Slides\n\
drive-mime-md = Markdown (.md)\n\
drive-mime-txt = Text\n\
drive-mime-file = File\n\
drive-save-inline = Save\n\
drive-open-doc-btn = Open Doc ↗\n\
drive-open-btn = Open ↗\n\
drive-preview-btn = Preview\n\
drive-rename-btn = Rename\n\
drive-restore-btn = Restore\n\
drive-delete-forever-btn = Delete Forever\n\
drive-trash-item-btn = Trash\n\
drive-confirm-perm-delete = Confirm permanent delete?\n\
drive-empty-files = No files found in this view. Click '+ New Google Doc' or save ADRs from Docs & ADRs.\n\
drive-close-preview = Close Preview\n\
drive-toast-doc-copied = Document content copied\n\
conn-msg-list-err = Unable to list Google Drive & Docs files.\n\
conn-msg-load-err = Unable to load connected apps.\n\
conn-msg-revoking = Disconnecting and revoking Google tokens...\n\
conn-msg-revoked = Google Drive & Docs disconnected and OAuth tokens revoked.\n\
conn-toast-revoked = Google Drive & Docs revoked\n\
conn-msg-revoke-err = Failed to revoke Google authorization.\n\
conn-msg-creating-folder = Creating folder...\n\
conn-toast-folder-created = Folder created in Google Drive\n\
conn-msg-folder-err = Folder create failed.\n\
conn-msg-doc-required = Enter both a title and Markdown content.\n\
conn-msg-converting-gdoc = Converting Markdown to native Google Doc...\n\
conn-msg-uploading-md = Uploading Markdown file to Google Drive...\n\
conn-toast-doc-saved = Document saved to Google Docs / Drive\n\
conn-msg-preview-err = Preview failed.\n\
conn-toast-item-updated = Drive item updated\n\
conn-msg-action-err = Action failed.\n\
tab-overview = Overview\n\
tab-works = Works\n\
tab-groups = Groups & Invites\n\
tab-wallet = Wallet\n\
tab-paywalls = Paywalls\n\
tab-plans = Plans\n\
tab-personas = Personas\n\
tab-langs-alerts = Languages & Notifications\n\
profile-tabs-aria = Profile sections\n\
profile-overview-title = Public Creator Identity Overview\n\
profile-overview-desc = Your sovereign root identity anchors your works, personas, group invitations, paywall payouts, and reader purchases.\n\
active-persona-label = Active persona\n\
profile-sync-plan-label = Personal sync plan:\n\
personas-label = Personas\n\
groups-label = Groups\n\
referrals-label = Referrals\n\
profile-referred-by = Referred to ExoNovel by:\n\
profile-personas-unit = workspace keys\n\
profile-groups-unit = collaborative groups\n\
profile-referrals-unit = joined writers\n\
works-title = Authored & Forked Works\n\
works-desc = Every manuscript stores collaborative Loro CRDT edits alongside Willow Meadowcap capability paths anchored to your active persona.\n\
works-novels-title = Novels & Serials\n\
works-novels-desc = Chapters and scenes saved with free public or paid reader tiers configured in Paywalls.\n\
works-scripts-title = Screenplays & Poetry\n\
works-scripts-desc = Multi-language input composer (29 languages + RTL) with live collaborative cursors.\n\
groups-title = Groups & Managers\n\
groups-composer-label = Group Composer\n\
groups-name-label = Group Name\n\
groups-name-ph = e.g. Anthology Editors\n\
groups-desc-label = Description\n\
groups-desc-ph = What this group collaborates on\n\
groups-create-btn = Create Group\n\
groups-invite-btn = Invite to Group\n\
groups-set-member-btn = Set as Member\n\
groups-assign-mgr-btn = Assign Manager\n\
invites-title = Invites & Referrals\n\
invites-sub = Invite writers and track joined referrals.\n\
invites-create-btn = Create Invite Link\n\
invites-th-scope = Invite Scope\n\
invites-th-link = Invite Link\n\
invites-th-uses = Uses\n\
invites-th-created = Created\n\
invites-scope-general = General ExoNovel Invite\n\
wallet-tab-title = Connected Wallets & Payment Rails\n\
wallet-tab-sub = Choose how you receive reader payments and pay for personal device sync plans across all supported local currency, card, and crypto rails.\n\
wallet-selected-rail-title = Active Settlement Rail\n\
wallet-selected-rail-desc = Used when activating personal sync plans or receiving writer payouts.\n\
wallet-card-desc = Cards, Apple Pay, Google Pay, and local regional currency settlements.\n\
wallet-solana-desc = Sub-second stablecoin creator payouts and instant reader unlocks.\n\
wallet-btc-desc = Low-fee Lightning invoices for global micro-unlocks and subscriptions.\n\
ledger-wallet-title = Wallet Transactions & Receipts\n\
ledger-wallet-sub = Complete record of writer earnings, reader unlock receipts, and sync plan charges.\n\
paywalls-title = Author Paywalls & Reader Access\n\
paywalls-sub = Decide which stories, chapters, or reusable worldbuilding blocks stay free for everyone and which unlock for paying readers—with automatic attribution splits for collaborators.\n\
paywalls-free-tier-title = Free Public Chapters\n\
paywalls-free-tier-desc = Keep opening chapters or whole books open to all readers so newcomers can discover your writing.\n\
paywalls-unlock-title = Per-Story & Chapter Paywalls\n\
paywalls-unlock-desc = Set direct reader unlock prices in your preferred currency across cards, mobile money, and crypto wallets.\n\
paywalls-splits-title = Collaborator Attribution Splits\n\
paywalls-splits-desc = When you co-write in a group, paywall revenue traces cleanly to each credited creator.\n\
paywalls-ledger-title = Reader Unlocks & Paywall Activity\n\
paywalls-ledger-sub = Works you unlocked as a reader and paid reader unlocks across your own paywalled stories.\n\
paywalls-empty = No paywall sales or reader unlock receipts recorded yet.\n\
plans-title = Writing Access & Personal Device Sync Plans\n\
plans-sub = Writing on the web is free, writing on your own devices is free, and team collaboration sync between writers is always free—even when one writer is on the web and another works offline on a device. A paid plan is only for keeping your own offline device drafts synced with your own web profile.\n\
plans-payment-rail-note = Selected Payment Rail\n\
plans-change-wallet-btn = Configure in Wallet →\n\
plans-free-title = Web, Device & Collaboration (Free)\n\
plans-free-desc = Free writing on the web, free local writing on your devices, and free collaboration sync between you and your writing team.\n\
wallet-peer-prefix = ExoNovel Service Recipient Peer:\n\
wallet-currency-label = Local Currency\n\
wallet-rail-label = Default Payment Rail (AOTA)\n\
wallet-cur-usd = USD ($9/mo · $90/yr)\n\
wallet-cur-eur = EUR (€9/mo · €90/yr)\n\
wallet-cur-gbp = GBP (£7/mo · £70/yr)\n\
wallet-cur-pln = PLN (36 zł/mo · 360 zł/yr)\n\
wallet-cur-ngn = NGN (₦13,500/mo · ₦135,000/yr)\n\
wallet-cur-kes = KES (KSh 1,200/mo · KSh 12,000/yr)\n\
wallet-cur-inr = INR (₹750/mo · ₹7,500/yr)\n\
wallet-cur-brl = BRL (R$45/mo · R$450/yr)\n\
wallet-rail-card = Card / Local Currency Gateway\n\
wallet-rail-solana = Solana Pay / USDC\n\
wallet-rail-btc = Bitcoin Lightning (L402)\n\
wallet-rail-eth = Base / Arbitrum / Ethereum\n\
wallet-rail-mobile = Mobile Money / PIX / UPI\n\
wallet-free-current = Current Plan (Free)\n\
wallet-free-switch = Switch to Free\n\
wallet-mo-title = Personal Sync Monthly ($9/mo eq.)\n\
wallet-mo-desc = Encrypted sync connecting your own offline device drafts with your own web profile.\n\
wallet-mo-active = Active Monthly\n\
wallet-mo-activate = Activate $9/month\n\
wallet-yr-title = Personal Sync Annual ($90/yr eq.)\n\
wallet-yr-desc = 2 months free · full encrypted sync between your own devices and your own web profile.\n\
wallet-yr-active = Active Annual\n\
wallet-yr-activate = Activate $90/year\n\
ledger-th-type = Type\n\
ledger-th-amount = Amount\n\
ledger-th-cycle = Cycle\n\
ledger-th-details = Details\n\
ledger-th-date = Date\n\
personas-title = Pen Names & Personas\n\
personas-sub = Each persona carries its own cryptographic subspace key and avatar while rolling up to your root identity.\n\
persona-default-writer = Writer\n\
persona-use-btn = Use\n\
persona-edit-btn = Edit Persona & Avatar\n\
persona-display-ph = Display name\n\
persona-handle-ph = Handle\n\
persona-bio-ph = Bio\n\
persona-save-btn = Save Persona\n\
persona-create-heading = Create New Persona\n\
persona-composer-label = Persona Composer\n\
persona-display-label = Persona Display Name\n\
persona-handle-label = Handle\n\
persona-bio-label = Bio\n\
persona-create-btn = Create Persona\n\
langs-quick-title = Personal Quick-Switch Languages\n\
langs-quick-sub = Choose which of the 29 supported languages appear in your compact top-bar UI switcher and your writing composer Input switcher. All 29 remain available in Preferences.\n\
langs-pinned-ui-label = Pinned Output UI Languages\n\
langs-pinned-input-label = Pinned Composer Input Languages\n\
langs-save-btn = Save Quick-Switch Language Lists\n\
activity-notif-title = Activity, Collaboration & Commercial Notifications\n\
activity-notif-sub = Choose which persona activity, group invitations, paywall earnings, and reader purchases generate notifications.\n\
notif-earn-title = Author Paywall Earnings\n\
notif-earn-desc = Notify when a reader unlocks your paid works.\n\
notif-purch-title = Reader Purchases & Receipts\n\
notif-purch-desc = Receipts when you purchase works or unlock paywalls from other authors.\n\
notif-inv-title = Group Invitations & Role Changes\n\
notif-inv-desc = Invites and Manager assignments.\n\
notif-ref-title = Referral Signups\n\
notif-ref-desc = Alerts when a writer joins via your personal invite link.\n\
notif-collab-title = Co-Author & Collaborator Notes\n\
notif-collab-desc = Inline edits and replies on shared works.\n\
notif-fork-title = Mentions & Forks\n\
notif-fork-desc = Alerts when your building blocks are forked with attribution.\n\
activity-notif-save-btn = Save Activity Notification Rules\n\
profile-toast-persona-created = Persona created\n\
profile-toast-persona-updated = Persona updated\n\
profile-msg-img-size = Image must be 2 MB or smaller.\n\
profile-toast-avatar-done = Persona avatar processed\n\
profile-toast-group-created = Group created\n\
profile-toast-role-updated = Member role updated\n\
profile-toast-invite-copied = Invite link copied\n\
profile-toast-plan-updated = Sync plan updated\n\
profile-toast-langs-saved = Language quick-switchers saved\n\
profile-toast-notif-saved = Activity notification preferences saved\n\
cms-all = All\n\
cms-on-hold = On Hold\n\
cms-ready = Ready\n\
cms-accepted = Accepted\n\
cms-proposed = Proposed\n\
cms-search-placeholder = Search ADRs...\n\
cms-sidebar-title = MADR 4.0.0 CMS\n\
cms-records-count-suffix = records\n\
cms-hide = Hide\n\
cms-show-explorer = Show Explorer\n\
cms-read-adr = Read ADR\n\
cms-new-madr = + New MADR 4.0\n\
cms-tooling-proposals = Tooling & Proposals\n\
cms-inference-runtimes = Inference Runtimes\n\
cms-owner-tasks = Owner & Ops Tasks\n\
cms-connect-google-docs = Connect Google Docs\n\
cms-backup-drive = Backup Archive to Drive\n\
cms-backup-title = Back up all ExoNovel Docs to Google Drive\n\
cms-backup-dest-default = Destination: ExoNovel Docs/\n\
cms-backup-dest-prefix = Destination: ExoNovel Docs/ on\n\
cms-backup-fmt-gdoc = Google Docs (Editable)\n\
cms-backup-fmt-md = Markdown (.md)\n\
cms-backup-start = Start Backup\n\
cms-drive-synced-suffix = is synced in Google Drive.\n\
cms-open-drive-btn = Open in Google Docs / Drive ↗\n\
cms-select-adr-prompt = Select a record from the left explorer.\n\
cms-words-suffix = words\n\
cms-min-read-suffix = min read\n\
cms-trans-canonical = Canonical (EN)\n\
cms-trans-translated = Translated\n\
cms-trans-fallback-en = English Fallback\n\
cms-save-gdoc-btn = Save to Google Docs\n\
cms-save-md-btn = Save .md to Drive\n\
cms-edit-adr-btn = Edit in MADR CMS\n\
cms-deps-title = Underlying Tooling Dependencies & Blockers\n\
cms-deps-empty = No blocking tooling dependencies attached to this ADR.\n\
cms-deps-installed = installed\n\
cms-deps-latest = latest\n\
cms-deps-req = req\n\
cms-deps-constraint-ph = >=0.4.0\n\
cms-deps-cap-ph = Blocking capability...\n\
cms-deps-satisfied = Satisfied\n\
cms-deps-link-btn = Link Dependency\n\
cms-editor-title-edit = Edit MADR 4.0.0 Record\n\
cms-editor-title-new = Create New MADR 4.0.0 ADR\n\
cms-editor-context-label = MADR Editor\n\
cms-field-title = Title\n\
cms-field-rel-path = Relative Path (e.g. docs/adr/010-short.md)\n\
cms-field-status = Status\n\
cms-field-makers = Decision Makers\n\
cms-field-context = Context and Problem Statement\n\
cms-field-drivers = Decision Drivers\n\
cms-field-outcome = Decision Outcome\n\
cms-field-raw-md = Full Markdown (optional override)\n\
cms-save-adr-btn = Save ADR\n\
cms-status-opt-proposed = proposed\n\
cms-status-opt-on-hold = on-hold (awaiting tooling)\n\
cms-status-opt-ready = ready-to-build\n\
cms-status-opt-accepted = accepted\n\
cms-status-opt-deprecated = deprecated\n\
cms-status-opt-superseded = superseded\n\
cms-status-opt-rejected = rejected\n\
cms-on-this-page = On this page\n\
cms-tooling-catalog-title = Tooling Catalog & Release Scanner\n\
cms-tooling-catalog-sub = Deterministic crates.io version monitoring linked to architectural decision records.\n\
cms-run-scan = Run crates.io Scan\n\
cms-th-tool = Tool\n\
cms-th-installed = Installed\n\
cms-th-latest = Latest\n\
cms-th-purpose = Purpose\n\
cms-add-tool-heading = Track Another Underlying Crate / Tool\n\
cms-add-tool-key-ph = Key (e.g. reqwest)\n\
cms-add-tool-name-ph = Display Name\n\
cms-add-tool-pkg-ph = crates.io package\n\
cms-add-tool-ver-ph = Installed version\n\
cms-add-tool-purpose-ph = Why ExoNovel depends on it\n\
cms-add-tool-btn = Add Tool\n\
cms-ai-providers-title = AI Inference Providers & Active Models\n\
cms-ai-providers-sub = Configured local CLI runtime and verified active provider models available in this environment.\n\
cms-refresh-models = Refresh Live Models\n\
cms-ai-empty = No active inference providers configured in this environment.\n\
cms-model-deprecated = Deprecated\n\
cms-model-cooldown = Cooldown\n\
cms-model-active = Active\n\
cms-maturity-proposals-title = Maturity Proposals\n\
cms-maturity-proposals-sub = Generate upgrade recommendations for tracked dependencies and accept or dismiss proposals.\n\
cms-generate-proposal = Generate Proposal Note\n\
cms-proposals-empty = No proposal notes in this filter yet. Select a tool above and click Generate Proposal Note.\n\
cms-proposal-filter-all = All\n\
cms-proposal-filter-open = Open\n\
cms-proposal-filter-accepted = Accepted\n\
cms-proposal-filter-dismissed = Dismissed\n\
cms-proposal-status-prefix = Status:\n\
cms-proposal-accept = Accept Proposal\n\
cms-proposal-dismiss = Dismiss\n\
cms-proposal-reopen = Reopen\n\
cms-proposal-rationale = Diagnostics / Rationale:\n\
cms-owner-tasks-title = External Owner & Ops Checklist\n\
cms-owner-tasks-sub = Tasks requiring external consoles or production server configuration. Check a box when done to persist completion state in the database.\n\
cms-task-done = Done\n\
cms-task-pending = Pending\n\
cms-msg-redirecting-google = Redirecting to Google consent screen...\n\
cms-msg-saving-gdoc = Saving active ADR to Google Docs...\n\
cms-msg-saving-md = Backing up active ADR as .md to Google Drive...\n\
cms-msg-save-failed = Save failed.\n\
cms-toast-saved-drive = Saved to Google Docs / Drive\n\
cms-msg-backing-up-archive = Backing up full ExoNovel Docs & ADR archive to Google Drive (this may take a moment)...\n\
cms-msg-archive-failed = Archive backup failed.\n\
cms-toast-archive-backed-up = Archive backed up to Google Drive\n\
cms-toast-task-done = Task marked complete\n\
cms-toast-task-reopened = Task reopened\n\
cms-toast-task-failed = Failed to update task\n\
cms-msg-saving-adr = Saving MADR 4.0.0 record...\n\
cms-msg-missing-token = Missing session token.\n\
cms-toast-adr-saved = ADR saved to CMS\n\
cms-toast-dep-linked = Tooling dependency linked to ADR\n\
cms-msg-scanning-crates = Running deterministic crates.io release scan...\n\
cms-toast-scan-done = crates.io release catalog updated\n\
cms-msg-scan-failed = Scan failed.\n\
cms-msg-refreshing-models = Refreshing live /v1/models across configured inference runtimes...\n\
cms-toast-models-refreshed = Live provider models refreshed\n\
cms-msg-generating-proposal = Running active inference proposal analysis...\n\
cms-toast-proposal-generated = Maturity proposal note generated\n\
cms-toast-proposal-updated = Proposal status updated\n\
cms-toast-tool-added = Tool added to Maturity Catalog\n\
cms-toast-copied = Copied text to clipboard\n\
nav-group-account = Account\n\
nav-group-workspace = Workspace & Delivery\n\
nav-group-admin = Administration\n\
nav-group-writing = Writing & Collaboration\n\
nav-group-commerce = Billing & Reader Access\n\
nav-group-personalization = Personalization\n\
settings-resize-aria = Resize settings navigation sidebar\n",
    }
}

pub fn render_ftl_pack(code: &str) -> Option<String> {
    let clean = code.strip_suffix(".ftl").unwrap_or(code).trim();
    let seed = LOCALES.iter().find(|l| l.code == clean)?;
    let extra = extended_ui_block(seed.code);
    Some(format!(
        "# ExoNovel Fluent Locale Pack ({code})\n\
         # Direction: {dir} | Willow Read-Only Blob: /public/i18n/{code}.ftl\n\n\
         nav-desk = {nav_desk}\n\
         nav-dear-world = {nav_dear_world}\n\
         nav-docs = {nav_docs}\n\
         nav-profile = {nav_profile}\n\
         nav-preferences = {nav_preferences}\n\
         nav-sign-out = {nav_sign_out}\n\
         ui-lang-label = {ui_lang_label}\n\
         input-lang-label = {input_lang_label}\n\
         live-sync-label = {live_sync_label}\n\
         profile-title = {profile_title}\n\
         preferences-title = {preferences_title}\n\
         notifications-title = {notifications_title}\n\
         save-label = {save_label}\n\
         back-to-desk = ← {nav_desk}\n\
         {extra}",
        code = seed.code,
        dir = seed.dir,
        nav_desk = seed.nav_desk,
        nav_dear_world = seed.nav_dear_world,
        nav_docs = seed.nav_docs,
        nav_profile = seed.nav_profile,
        nav_preferences = seed.nav_preferences,
        nav_sign_out = seed.nav_sign_out,
        ui_lang_label = seed.ui_lang_label,
        input_lang_label = seed.input_lang_label,
        live_sync_label = seed.live_sync_label,
        profile_title = seed.profile_title,
        preferences_title = seed.preferences_title,
        notifications_title = seed.notifications_title,
        save_label = seed.save_label,
        extra = extra,
    ))
}


pub async fn api_i18n_catalog() -> impl IntoResponse {
    Json(serde_json::json!({
        "count": LOCALES.len(),
        "publicWillowNamespace": "exonovel-public-assets",
        "locales": catalog_list(),
    }))
}

pub async fn api_i18n_pack(Path(locale): Path<String>) -> impl IntoResponse {
    let Some(ftl) = render_ftl_pack(&locale) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Locale pack not found." })),
        )
            .into_response();
    };
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=3600"),
    );
    (StatusCode::OK, headers, ftl).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn extract_ftl_keys(pack: &str) -> BTreeSet<String> {
        pack.lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    return None;
                }
                trimmed.split_once('=').map(|(k, _)| k.trim().to_string())
            })
            .collect()
    }

    #[test]
    fn exposes_29_locales_with_rtl_arabic_and_hebrew() {
        let list = catalog_list();
        assert_eq!(list.len(), 29);
        assert_eq!(direction_for_locale("ar"), "rtl");
        assert_eq!(direction_for_locale("he"), "rtl");
        assert_eq!(direction_for_locale("uk"), "ltr");
        assert!(render_ftl_pack("zu.ftl").is_some());
    }

    #[test]
    fn ui_packs_have_identical_keys_across_en_de_pl() {
        let en_keys = extract_ftl_keys(&render_ftl_pack("en").expect("en pack"));
        let de_keys = extract_ftl_keys(&render_ftl_pack("de").expect("de pack"));
        let pl_keys = extract_ftl_keys(&render_ftl_pack("pl").expect("pl pack"));
        assert_eq!(en_keys, de_keys, "de FTL keys diverge from en");
        assert_eq!(en_keys, pl_keys, "pl FTL keys diverge from en");
    }
}
