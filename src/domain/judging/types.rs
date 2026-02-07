use serde::{Deserialize, Serialize};

#[cfg(feature = "server")]
use utoipa::ToSchema;

// ============================================================================
// Walk Type Enum
// ============================================================================

/// Type of random walk algorithm for judge routing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub enum WalkType {
    /// Default 2-phase algorithm: random selection for under-visited, softmax for rest
    #[default]
    Default,
    /// Proximity-based: routes to nearest table (L1 distance)
    Proximity,
}

impl WalkType {
    /// Convert from database string representation
    pub fn from_str(s: &str) -> Self {
        match s {
            "Proximity" => WalkType::Proximity,
            _ => WalkType::Default,
        }
    }

    /// Convert to database string representation
    pub fn to_str(&self) -> &'static str {
        match self {
            WalkType::Default => "Default",
            WalkType::Proximity => "Proximity",
        }
    }
}

/// Current assignment for a judge
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct JudgeAssignment {
    pub visit_id: i32,
    pub submission_id: i32,
    pub team_name: String,
    pub submission_data: serde_json::Value,
    pub start_time: String,
    pub time_remaining_seconds: i64,
}

/// Assignment for comparing two projects
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct ComparisonAssignment {
    pub feature: FeatureInfo,
    pub submission_a: SubmissionSummary,
    pub submission_b: SubmissionSummary,
}

/// Request payload for completing a visit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct CompleteVisitRequest {
    pub notes: Option<String>,
}

/// Request payload for submitting a pairwise comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct PairwiseComparisonRequest {
    pub feature_id: i32,
    pub submission_a_id: i32,
    pub submission_b_id: i32,
    /// ID of the winning submission, or None for a tie
    pub winner_id: Option<i32>,
}

/// Judging status for a hackathon
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct JudgingStatus {
    pub submissions_closed: bool,
    pub judging_started: bool,
    pub total_submissions: i64,
    pub visited_submissions: i64,
    pub total_visits: i64,
    pub total_comparisons: i64,
    pub projects_with_tables: i64,
    pub unassigned_projects: Vec<String>,
}

/// Feature definition for judging
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct FeatureInfo {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

/// Project ranking entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct ProjectRanking {
    pub submission_id: i32,
    pub team_name: String,
    pub score: Option<f32>,
    pub visit_count: i32,
}

/// Rankings per prize track
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct PrizeRankings {
    pub prize_id: i32,
    pub prize_name: String,
    pub rankings: Vec<ProjectRanking>,
}

/// Request to create a feature
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct CreateFeatureRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Request to update a feature
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct UpdateFeatureRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Visit information for admin dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct VisitInfo {
    pub id: i32,
    pub submission_id: i32,
    pub team_name: String,
    pub judge_name: String,
    pub notes: Option<String>,
    pub start_time: String,
    pub completion_time: Option<String>,
    pub is_active: bool,
}

/// AI summary for a submission
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct SubmissionSummary {
    pub submission_id: i32,
    pub team_name: String,
    pub summary: Option<String>,
    pub visit_count: i32,
}

// ============================================================================
// Unified Judging Mode Types
// ============================================================================

/// Judge's assignment to a feature with their current best pick
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct JudgeFeatureState {
    pub feature_id: i32,
    pub feature_name: String,
    pub feature_description: Option<String>,
    pub current_best_submission_id: Option<i32>,
    pub current_best_team_name: Option<String>,
    pub current_best_description: Option<String>,
    pub current_best_table_number: Option<String>,
    pub notes: Option<String>,
}

/// Full state for the unified judging interface
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct UnifiedJudgingState {
    pub current_project: Option<CurrentProject>,
    pub features: Vec<JudgeFeatureState>,
    pub assigned_prizes: Vec<PrizeInfo>,
    pub all_prizes: Vec<PrizeInfo>,
    pub judging_started: bool,
    /// The judge's current walk type preference
    pub walk_type: WalkType,
}

/// Current project being judged
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct CurrentProject {
    pub visit_id: i32,
    pub submission_id: i32,
    pub team_name: String,
    pub project_name: Option<String>,
    pub location: Option<String>,
    pub table_number: Option<String>,
    pub description: Option<String>,
    pub submission_data: serde_json::Value,
    /// When the judge started visiting this project (for timer calculation)
    pub start_time: chrono::NaiveDateTime,
}

/// Request to submit comparisons for all features at once
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct SubmitComparisonsRequest {
    pub visit_id: i32,
    pub comparisons: Vec<FeatureComparison>,
    pub notes: Option<String>,
}

/// A single feature comparison decision
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct FeatureComparison {
    pub feature_id: i32,
    /// ID of the winning submission (current project or best-so-far)
    pub winner_submission_id: i32,
    pub notes: Option<String>,
}

/// Request to assign judges to a feature
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct AssignJudgesRequest {
    pub judge_ids: Vec<i32>,
}

/// Info about a judge assigned to a feature (for admin view)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct JudgeInfo {
    pub user_id: i32,
    pub name: String,
    pub email: Option<String>,
}

/// Feature with assigned judges (for admin view)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct FeatureWithJudges {
    pub feature: FeatureInfo,
    pub judges: Vec<JudgeInfo>,
}

/// Prize track info
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct PrizeInfo {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

/// Prize track with assigned judges (for admin view)
/// If judges is empty, this is a "default" track (all judges can judge)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct PrizeWithJudges {
    pub prize: PrizeInfo,
    pub judges: Vec<JudgeInfo>,
    pub is_default: bool,
}

// ============================================================================
// Results Page Types
// ============================================================================

/// Feature rank for a project in the results view
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct FeatureRankInfo {
    pub feature_id: i32,
    pub feature_name: String,
    pub rank: Option<i32>,
}

/// Results data for a project in a prize track
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct ProjectResultInfo {
    pub submission_id: i32,
    pub project_name: Option<String>,
    pub team_name: String,
    pub weighted_score: Option<f32>,
    pub rank: i32,
    pub table_number: Option<String>,
    pub feature_ranks: Vec<FeatureRankInfo>,
    pub description: Option<String>,
    pub repo_url: Option<String>,
    pub presentation_url: Option<String>,
    pub video_url: Option<String>,
    pub ai_summary: Option<String>,
    pub submission_data: Option<serde_json::Value>,
}

/// Full results for a prize track
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct PrizeTrackResults {
    pub prize_id: i32,
    pub prize_name: String,
    pub features: Vec<FeatureInfo>,
    pub projects: Vec<ProjectResultInfo>,
}

/// Judge's notes on a specific project (from their visit)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct JudgeVisitNotes {
    pub visited: bool,
    pub notes: Option<String>,
}

/// Response containing a generated AI summary
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct AiSummaryResponse {
    pub summary: String,
}

/// Response containing AI-generated suggested questions for judges
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(ToSchema))]
pub struct AiQuestionsResponse {
    pub questions: Vec<String>,
}
