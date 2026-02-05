use super::super::types::*;

/// Create the TartanHacks application form preset
pub fn tartanhacks_preset() -> FormSchema {
    let mut order = 0;
    let mut next_order = || {
        let o = order;
        order += 1;
        o
    };

    FormSchema {
        title: "TartanHacks Application".to_string(),
        description: Some("Apply to participate in TartanHacks".to_string()),
        fields: vec![
            // Basic Information
            FormField {
                id: "first_name".to_string(),
                field_type: FieldType::Text {
                    placeholder: None,
                    validation: None,
                },
                label: "First Name".to_string(),
                name: "first_name".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Personal Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "middle_name".to_string(),
                field_type: FieldType::Text {
                    placeholder: None,
                    validation: None,
                },
                label: "Middle Name".to_string(),
                name: "middle_name".to_string(),
                required: false,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Personal Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "last_name".to_string(),
                field_type: FieldType::Text {
                    placeholder: None,
                    validation: None,
                },
                label: "Last Name".to_string(),
                name: "last_name".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Personal Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "age".to_string(),
                field_type: FieldType::Number {
                    placeholder: None,
                    validation: Some(NumberValidation {
                        min: Some(13.0),
                        max: Some(120.0),
                        step: Some(1.0),
                    }),
                },
                label: "Age".to_string(),
                name: "age".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Personal Information".to_string()),
                conditional: None,
            },
            // Gender
            FormField {
                id: "gender".to_string(),
                field_type: FieldType::Radio {
                    options: vec![
                        SelectOption {
                            label: "Woman".to_string(),
                            value: "woman".to_string(),
                        },
                        SelectOption {
                            label: "Man".to_string(),
                            value: "man".to_string(),
                        },
                        SelectOption {
                            label: "Non-binary".to_string(),
                            value: "non_binary".to_string(),
                        },
                        SelectOption {
                            label: "Prefer to self-describe".to_string(),
                            value: "self_describe".to_string(),
                        },
                        SelectOption {
                            label: "Prefer not to say".to_string(),
                            value: "prefer_not_to_say".to_string(),
                        },
                    ],
                },
                label: "What is your gender?".to_string(),
                name: "gender".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Personal Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "gender_self_describe".to_string(),
                field_type: FieldType::Text {
                    placeholder: None,
                    validation: None,
                },
                label: "If you prefer to self-describe, please specify".to_string(),
                name: "gender_self_describe".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Personal Information".to_string()),
                conditional: Some(FieldCondition {
                    field: "gender".to_string(),
                    value: vec!["self_describe".to_string()],
                }),
            },
            // Race and Ethnicity
            FormField {
                id: "race_ethnicity".to_string(),
                field_type: FieldType::CheckboxGroup {
                    options: vec![
                        SelectOption {
                            label:
                                "Black / African American (including Sub-Saharan African diaspora)"
                                    .to_string(),
                            value: "black".to_string(),
                        },
                        SelectOption {
                            label:
                                "Middle Eastern / North African (e.g., Egyptian, Lebanese, Iranian)"
                                    .to_string(),
                            value: "middle_eastern".to_string(),
                        },
                        SelectOption {
                            label: "East Asian (e.g., Chinese, Korean, Japanese)".to_string(),
                            value: "east_asian".to_string(),
                        },
                        SelectOption {
                            label: "South Asian (e.g., Indian, Pakistani, Bangladeshi)".to_string(),
                            value: "south_asian".to_string(),
                        },
                        SelectOption {
                            label: "Southeast Asian (e.g. Filipino, Vietnamese, Thai, Indonesian)"
                                .to_string(),
                            value: "southeast_asian".to_string(),
                        },
                        SelectOption {
                            label: "Hispanic / Latine".to_string(),
                            value: "hispanic".to_string(),
                        },
                        SelectOption {
                            label: "Native American / Alaska Native".to_string(),
                            value: "native_american".to_string(),
                        },
                        SelectOption {
                            label: "Pacific Islander / Native Hawaiian".to_string(),
                            value: "pacific_islander".to_string(),
                        },
                        SelectOption {
                            label: "White / European".to_string(),
                            value: "white".to_string(),
                        },
                        SelectOption {
                            label: "Prefer to self-describe".to_string(),
                            value: "self_describe".to_string(),
                        },
                        SelectOption {
                            label: "Prefer not to say".to_string(),
                            value: "prefer_not_to_say".to_string(),
                        },
                    ],
                },
                label: "Please select your race/ethnicity (select all that apply)".to_string(),
                name: "race_ethnicity".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Personal Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "race_self_describe_text".to_string(),
                field_type: FieldType::Text {
                    placeholder: None,
                    validation: None,
                },
                label: "If you prefer to self-describe, please specify".to_string(),
                name: "race_self_describe_text".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Personal Information".to_string()),
                conditional: Some(FieldCondition {
                    field: "race_ethnicity".to_string(),
                    value: vec!["self_describe".to_string()],
                }),
            },
            // Region
            FormField {
                id: "state".to_string(),
                field_type: FieldType::Text {
                    placeholder: None,
                    validation: None,
                },
                label: "State".to_string(),
                name: "state".to_string(),
                required: false,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Geographic".to_string()),
                conditional: None,
            },
            FormField {
                id: "country".to_string(),
                field_type: FieldType::Text {
                    placeholder: None,
                    validation: None,
                },
                label: "Country".to_string(),
                name: "country".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Geographic".to_string()),
                conditional: None,
            },
            // Academic
            FormField {
                id: "school".to_string(),
                field_type: FieldType::Text {
                    placeholder: None,
                    validation: None,
                },
                label: "School".to_string(),
                name: "school".to_string(),
                required: true,
                help_text: Some("Please write out the full name with no abbreviations".to_string()),
                default_value: None,
                order: next_order(),
                section: Some("Academic Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "college".to_string(),
                field_type: FieldType::Select {
                    options: vec![
                        SelectOption {
                            label: "School of Computer Science".to_string(),
                            value: "scs".to_string(),
                        },
                        SelectOption {
                            label: "Carnegie Institute of Technology".to_string(),
                            value: "cit".to_string(),
                        },
                        SelectOption {
                            label: "College of Fine Arts".to_string(),
                            value: "cfa".to_string(),
                        },
                        SelectOption {
                            label: "Dietrich College of Humanities and Social Sciences".to_string(),
                            value: "dietrich".to_string(),
                        },
                        SelectOption {
                            label: "Mellon College of Science".to_string(),
                            value: "mcs".to_string(),
                        },
                        SelectOption {
                            label: "Tepper School of Business".to_string(),
                            value: "tepper".to_string(),
                        },
                        SelectOption {
                            label: "Heinz College".to_string(),
                            value: "heinz".to_string(),
                        },
                    ],
                    placeholder: Some("Select your college".to_string()),
                },
                label: "College (if CMU)".to_string(),
                name: "college".to_string(),
                required: false,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Academic Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "academic_program".to_string(),
                field_type: FieldType::Select {
                    options: vec![
                        SelectOption {
                            label: "Undergraduate".to_string(),
                            value: "undergrad".to_string(),
                        },
                        SelectOption {
                            label: "Masters".to_string(),
                            value: "masters".to_string(),
                        },
                        SelectOption {
                            label: "Doctorate".to_string(),
                            value: "doctorate".to_string(),
                        },
                        SelectOption {
                            label: "Other".to_string(),
                            value: "other".to_string(),
                        },
                    ],
                    placeholder: Some("Select your program".to_string()),
                },
                label: "Academic Program".to_string(),
                name: "academic_program".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Academic Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "academic_program_other".to_string(),
                field_type: FieldType::Text {
                    placeholder: None,
                    validation: None,
                },
                label: "If other, please specify".to_string(),
                name: "academic_program_other".to_string(),
                required: false,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Academic Information".to_string()),
                conditional: Some(FieldCondition {
                    field: "academic_program".to_string(),
                    value: vec!["other".to_string()],
                }),
            },
            FormField {
                id: "graduation_year".to_string(),
                field_type: FieldType::Select {
                    options: vec![
                        SelectOption {
                            label: "2026".to_string(),
                            value: "2026".to_string(),
                        },
                        SelectOption {
                            label: "2027".to_string(),
                            value: "2027".to_string(),
                        },
                        SelectOption {
                            label: "2028".to_string(),
                            value: "2028".to_string(),
                        },
                        SelectOption {
                            label: "2029".to_string(),
                            value: "2029".to_string(),
                        },
                        SelectOption {
                            label: "2030".to_string(),
                            value: "2030".to_string(),
                        },
                    ],
                    placeholder: Some("Select your graduation year".to_string()),
                },
                label: "Graduation Year".to_string(),
                name: "graduation_year".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Academic Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "major".to_string(),
                field_type: FieldType::Text {
                    placeholder: None,
                    validation: None,
                },
                label: "Major".to_string(),
                name: "major".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Academic Information".to_string()),
                conditional: None,
            },
            // Hackathon-related
            FormField {
                id: "hackathon_experience".to_string(),
                field_type: FieldType::Select {
                    options: vec![
                        SelectOption {
                            label: "0".to_string(),
                            value: "0".to_string(),
                        },
                        SelectOption {
                            label: "1-3".to_string(),
                            value: "1-3".to_string(),
                        },
                        SelectOption {
                            label: "4+".to_string(),
                            value: "4+".to_string(),
                        },
                    ],
                    placeholder: Some("Select your experience level".to_string()),
                },
                label: "Years of hackathon experience".to_string(),
                name: "hackathon_experience".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Academic Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "resume".to_string(),
                field_type: FieldType::File {
                    file_path: "{hackathon_slug}/applications/{user_oidc_sub}/resume".to_string(),
                    validation: Some(FileValidation {
                        accept: Some(".pdf,.doc,.docx".to_string()),
                        max_size: Some(10 * 1024 * 1024), // 10MB
                        multiple: false,
                    }),
                },
                label: "Resume".to_string(),
                name: "resume".to_string(),
                required: true,
                help_text: Some("Please upload your resume (max 10 MB)".to_string()),
                default_value: None,
                order: next_order(),
                section: Some("Portfolio Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "github_url".to_string(),
                field_type: FieldType::Url {
                    placeholder: Some("https://github.com/username".to_string()),
                },
                label: "GitHub/GitLab/etc. profile URL".to_string(),
                name: "github_url".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Portfolio Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "linkedin_url".to_string(),
                field_type: FieldType::Url {
                    placeholder: Some("https://linkedin.com/in/username".to_string()),
                },
                label: "LinkedIn profile URL".to_string(),
                name: "linkedin_url".to_string(),
                required: false,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Portfolio Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "website".to_string(),
                field_type: FieldType::Url {
                    placeholder: Some("https://example.com".to_string()),
                },
                label: "Website".to_string(),
                name: "website".to_string(),
                required: false,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Portfolio Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "design_portfolio".to_string(),
                field_type: FieldType::Url {
                    placeholder: Some("https://behance.net/username".to_string()),
                },
                label: "Design portfolio".to_string(),
                name: "design_portfolio".to_string(),
                required: false,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Portfolio Information".to_string()),
                conditional: None,
            },
            // Logistics
            FormField {
                id: "phone".to_string(),
                field_type: FieldType::Tel { placeholder: None },
                label: "Phone number".to_string(),
                name: "phone".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Personal Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "dietary_restrictions".to_string(),
                field_type: FieldType::CheckboxGroup {
                    options: vec![
                        SelectOption {
                            label: "Vegetarian".to_string(),
                            value: "vegetarian".to_string(),
                        },
                        SelectOption {
                            label: "Vegan".to_string(),
                            value: "vegan".to_string(),
                        },
                        SelectOption {
                            label: "Gluten-free".to_string(),
                            value: "gluten_free".to_string(),
                        },
                        SelectOption {
                            label: "Dairy-free".to_string(),
                            value: "dairy_free".to_string(),
                        },
                        SelectOption {
                            label: "Nut allergy".to_string(),
                            value: "nut_allergy".to_string(),
                        },
                        SelectOption {
                            label: "Shellfish allergy".to_string(),
                            value: "shellfish_allergy".to_string(),
                        },
                        SelectOption {
                            label: "Halal".to_string(),
                            value: "halal".to_string(),
                        },
                        SelectOption {
                            label: "Kosher".to_string(),
                            value: "kosher".to_string(),
                        },
                        SelectOption {
                            label: "Other".to_string(),
                            value: "other".to_string(),
                        },
                    ],
                },
                label: "Dietary restrictions (select all that apply)".to_string(),
                name: "dietary_restrictions".to_string(),
                required: false,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Logistics".to_string()),
                conditional: None,
            },
            FormField {
                id: "dietary_restrictions_other".to_string(),
                field_type: FieldType::Text {
                    placeholder: None,
                    validation: None,
                },
                label: "If other, please specify".to_string(),
                name: "dietary_restrictions_other".to_string(),
                required: false,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Logistics".to_string()),
                conditional: Some(FieldCondition {
                    field: "dietary_restrictions".to_string(),
                    value: vec!["other".to_string()],
                }),
            },
            FormField {
                id: "shirt_size".to_string(),
                field_type: FieldType::Select {
                    options: vec![
                        SelectOption {
                            label: "XS".to_string(),
                            value: "xs".to_string(),
                        },
                        SelectOption {
                            label: "S".to_string(),
                            value: "s".to_string(),
                        },
                        SelectOption {
                            label: "M".to_string(),
                            value: "m".to_string(),
                        },
                        SelectOption {
                            label: "L".to_string(),
                            value: "l".to_string(),
                        },
                        SelectOption {
                            label: "XL".to_string(),
                            value: "xl".to_string(),
                        },
                        SelectOption {
                            label: "XXL".to_string(),
                            value: "xxl".to_string(),
                        },
                    ],
                    placeholder: Some("Select your size".to_string()),
                },
                label: "Shirt size".to_string(),
                name: "shirt_size".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Logistics".to_string()),
                conditional: None,
            },
            FormField {
                id: "will_use_hardware".to_string(),
                field_type: FieldType::Checkbox {
                    option: SelectOption {
                        label: "I will use hardware at this event".to_string(),
                        value: "true".to_string(),
                    },
                },
                label: "Hardware Usage".to_string(),
                name: "will_use_hardware".to_string(),
                required: false,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Logistics".to_string()),
                conditional: None,
            },
            // Travel
            FormField {
                id: "travel_reimbursement".to_string(),
                field_type: FieldType::Checkbox {
                    option: SelectOption {
                        label: "I would like to apply for travel reimbursement".to_string(),
                        value: "true".to_string(),
                    },
                },
                label: "Travel Reimbursement".to_string(),
                name: "travel_reimbursement".to_string(),
                required: false,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Travel".to_string()),
                conditional: None,
            },
            FormField {
                id: "travel_location".to_string(),
                field_type: FieldType::Text {
                    placeholder: Some("e.g., New York, NY".to_string()),
                    validation: None,
                },
                label: "Where are you traveling from?".to_string(),
                name: "travel_location".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Travel".to_string()),
                conditional: Some(FieldCondition {
                    field: "travel_reimbursement".to_string(),
                    value: vec!["true".to_string()],
                }),
            },
            FormField {
                id: "diversity_statement".to_string(),
                field_type: FieldType::Textarea { placeholder: None },
                label: "Statement of diversity".to_string(),
                name: "diversity_statement".to_string(),
                required: false,
                help_text: Some("This optional statement is used to help determine travel reimbursement eligibility. Please share any aspects of your background, experiences, or perspectives that contribute to diversity.".to_string()),
                default_value: None,
                order: next_order(),
                section: Some("Travel".to_string()),
                conditional: None,
            },
            FormField {
                id: "additional_notes".to_string(),
                field_type: FieldType::Textarea { placeholder: None },
                label: "Additional notes".to_string(),
                name: "additional_notes".to_string(),
                required: false,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Additional Notes".to_string()),
                conditional: None,
            },
            // Sponsor Information
            FormField {
                id: "us_work_authorization".to_string(),
                field_type: FieldType::Select {
                    options: vec![
                        SelectOption {
                            label: "I am a US citizen".to_string(),
                            value: "us_citizen".to_string(),
                        },
                        SelectOption {
                            label: "I will need employer sponsorship at some point in the future".to_string(),
                            value: "will_need_sponsorship".to_string(),
                        },
                        SelectOption {
                            label: "I will NOT need employer sponsorship at some point in the future".to_string(),
                            value: "no_sponsorship_needed".to_string(),
                        },
                    ],
                    placeholder: Some("Select your work authorization status".to_string()),
                },
                label: "US Work Authorization".to_string(),
                name: "us_work_authorization".to_string(),
                required: false,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Sponsor Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "work_location_preferences".to_string(),
                field_type: FieldType::Text {
                    placeholder: None,
                    validation: None,
                },
                label: "Work Location Preferences".to_string(),
                name: "work_location_preferences".to_string(),
                required: false,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Sponsor Information".to_string()),
                conditional: None,
            },
            // Consent
            FormField {
                id: "code_of_conduct".to_string(),
                field_type: FieldType::Checkbox {
                    option: SelectOption {
                        label: "I agree to abide by the TartanHacks Code of Conduct".to_string(),
                        value: "true".to_string(),
                    },
                },
                label: "THX Code of Conduct".to_string(),
                name: "code_of_conduct".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Consent Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "media_release".to_string(),
                field_type: FieldType::Checkbox {
                    option: SelectOption {
                        label: "I consent to being photographed/recorded at TartanHacks".to_string(),
                        value: "true".to_string(),
                    },
                },
                label: "Media release".to_string(),
                name: "media_release".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Consent Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "sponsor_info".to_string(),
                field_type: FieldType::Checkbox {
                    option: SelectOption {
                        label: "I authorize TartanHacks to send my information to sponsors".to_string(),
                        value: "true".to_string(),
                    },
                },
                label: "Sponsor Information Authorization".to_string(),
                name: "sponsor_info".to_string(),
                required: false,
                help_text: None,
                default_value: Some("true".to_string()),
                order: next_order(),
                section: Some("Consent Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "signature".to_string(),
                field_type: FieldType::Text {
                    placeholder: Some("Type your full name".to_string()),
                    validation: None,
                },
                label: "Signature".to_string(),
                name: "signature".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Consent Information".to_string()),
                conditional: None,
            },
            FormField {
                id: "signature_date".to_string(),
                field_type: FieldType::Date,
                label: "Date".to_string(),
                name: "signature_date".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Consent Information".to_string()),
                conditional: None,
            },
        ],
        version: "1.0".to_string(),
    }
}

pub fn tartanhacks_submission_preset() -> FormSchema {
    let mut order = 0;
    let mut next_order = || {
        let o = order;
        order += 1;
        o
    };

    FormSchema {
        title: "Project Submission".to_string(),
        description: Some("This is the information the judges will use to evaluate your project during deliberations!".to_string()),
        fields: vec![
            // Project Name
            FormField {
                id: "project_name".to_string(),
                field_type: FieldType::Text {
                    placeholder: Some("Enter your project name".to_string()),
                    validation: None,
                },
                label: "Project Name".to_string(),
                name: "project_name".to_string(),
                required: true,
                help_text: None,
                default_value: None,
                order: next_order(),
                section: Some("Project Details".to_string()),
                conditional: None,
            },
            // Project Description
            FormField {
                id: "project_description".to_string(),
                field_type: FieldType::Textarea {
                    placeholder: Some("Add a description about your project".to_string()),
                },
                label: "Project Description".to_string(),
                name: "project_description".to_string(),
                required: true,
                help_text: Some("Describe what your project does, the problem it solves, and any notable features.".to_string()),
                default_value: None,
                order: next_order(),
                section: Some("Project Details".to_string()),
                conditional: None,
            },
            // Repo URL
            FormField {
                id: "repo_url".to_string(),
                field_type: FieldType::Url {
                    placeholder: Some("https://github.com/username/project".to_string()),
                },
                label: "Repo URL".to_string(),
                name: "repo_url".to_string(),
                required: true,
                help_text: Some("Link to your project's source code repository.".to_string()),
                default_value: None,
                order: next_order(),
                section: Some("Project Links".to_string()),
                conditional: None,
            },
            // Zipped File Link (optional)
            FormField {
                id: "project_zip_url".to_string(),
                field_type: FieldType::Url {
                    placeholder: Some("https://drive.google.com/file/...".to_string()),
                },
                label: "Zipped File Link (optional)".to_string(),
                name: "project_zip_url".to_string(),
                required: false,
                help_text: Some("Link to a zip file of your project (e.g., Google Drive, Dropbox).".to_string()),
                default_value: None,
                order: next_order(),
                section: Some("Project Links".to_string()),
                conditional: None,
            },
            // Presentation URL
            FormField {
                id: "presentation_url".to_string(),
                field_type: FieldType::Url {
                    placeholder: Some("https://docs.google.com/presentation/...".to_string()),
                },
                label: "Presentation URL".to_string(),
                name: "presentation_url".to_string(),
                required: false,
                help_text: Some("Link to your project presentation slides.".to_string()),
                default_value: None,
                order: next_order(),
                section: Some("Project Links".to_string()),
                conditional: None,
            },
            // Video URL
            FormField {
                id: "video_url".to_string(),
                field_type: FieldType::Url {
                    placeholder: Some("https://youtube.com/watch?v=...".to_string()),
                },
                label: "Video URL".to_string(),
                name: "video_url".to_string(),
                required: false,
                help_text: Some("Link to a demo video of your project.".to_string()),
                default_value: None,
                order: next_order(),
                section: Some("Project Links".to_string()),
                conditional: None,
            },
            // Demo URL
            FormField {
                id: "demo_url".to_string(),
                field_type: FieldType::Url {
                    placeholder: Some("https://demo.terrier.scottylabs.org/...".to_string()),
                },
                label: "Demo URL".to_string(),
                name: "demo_url".to_string(),
                required: false,
                help_text: Some("Link to a demo of your project.".to_string()),
                default_value: None,
                order: next_order(),
                section: Some("Project Links".to_string()),
                conditional: None,
            },
        ],
        version: "1.0".to_string(),
    }
}

pub fn tartanhacks_apple_app_site_association() -> String {
    let content = "{
    \"applinks\": {
        \"apps\": [],
        \"details\": [
            {
                \"appIDs\": [
                    \"4Y39FMA838.org.scottylabs.slapp\",
                    \"X39S5JJUD8.org.scottylabs.slapp\",
                    \"4Y39FMA838.terrier.scottylabs.org\",
                    \"X39S5JJUD8.terrier.scottylabs.org\"
                ],
                \"paths\": [
                    \"/auth/*\",
                    \"/h/*\",
                    \"/h/*/scan/*\"
                ],
                \"components\": [
                    {
                        \"/\": \"/auth/*\",
                        \"comment\": \"OAuth callback - opens app for auth completion\"
                    },
                    {
                        \"/\": \"/h/*\",
                        \"comment\": \"Hackathon pages - deep link to specific hackathon\"
                    },
                    {
                        \"/\": \"/h/*/scan/*\",
                        \"comment\": \"QR scan check-in - deep link for organizer check-in\"
                    }
                ]
            }
        ]
    },
    \"webcredentials\": {
        \"apps\": [
            \"4Y39FMA838.org.scottylabs.slapp\",
            \"X39S5JJUD8.org.scottylabs.slapp\",
            \"4Y39FMA838.terrier.scottylabs.org\",
            \"X39S5JJUD8.terrier.scottylabs.org\"
        ]
    }
}";

    content.to_string()
}

pub fn tartanhacks_assetlinks_json() -> String {
    let content = r#"[
  {
    "relation": [
      "delegate_permission/common.handle_all_urls",
      "delegate_permission/common.get_accounts",
      "delegate_permission/common.use_as_origin"
    ],
    "target": {
      "namespace": "android_app",
      "package_name": "org.scottylabs.thdapp",
      "sha256_cert_fingerprints": [
        "95:7C:2E:2C:AD:EB:61:8B:F9:22:4E:16:28:D1:8F:FD:25:11:CF:3B:76:CD:F1:F4:BB:39:FB:A8:E2:A5:CE:76"
      ]
    }
  }
]"#;
    content.to_string()
}
