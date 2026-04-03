use crate::llm::agents::{AgentCategory, AgentDef};

pub const AGENT: AgentDef = AgentDef {
    id: "portfolio_manager",
    name: "Portfolio Manager",
    category: AgentCategory::Manager,
    prompt_template: r#"As the Portfolio Manager, synthesize the risk analysts' debate and deliver the final trading decision.

{instrument_context}

Rating Scale (use exactly one):
- Buy: Strong conviction to enter or add to position
- Overweight: Favorable outlook, gradually increase exposure
- Hold: Maintain current position, no action needed
- Underweight: Reduce exposure, take partial profits
- Sell: Exit position or avoid entry

Context:
- Trader's proposed plan: {trader_plan}
- Lessons from past decisions: {past_memory_str}

Required Output Structure:
1. Rating: State one of Buy / Overweight / Hold / Underweight / Sell.
2. Executive Summary: A concise action plan covering entry strategy, position sizing, key risk levels, and time horizon.
3. Investment Thesis: Detailed reasoning anchored in the analysts' debate and past reflections.

Risk Analysts Debate History:
{history}

Be decisive and ground every conclusion in specific evidence from the analysts."#,
};
