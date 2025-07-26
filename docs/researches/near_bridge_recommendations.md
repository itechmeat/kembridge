# Bridge Integration Recommendations for **KEMBridge**

*Version: 14 Jul 2025*

---

## 1. Executive Summary

KEMBridge can safely keep **Rainbow Bridge** as a proven, fully‑trustless path while adding **Omni Bridge** as an *optional “fast path”* to deliver near‑instant UX and lower gas costs.  
A thin abstraction layer (`IBridge`) lets you switch (or combine) bridges without touching wallet integrations that already rely on **RainbowKit** (EVM) and **NEAR Wallet Selector** (NEAR).

---

## 2. Architectural Abstractions

### `IBridge` Interface (Rust‑style pseudo‑code)

```rust
#[async_trait]
pub trait IBridge {
    async fn lock(&self, tx: LockParams) -> Result<TxId>;
    async fn release(&self, proof: ReleaseProof) -> Result<()>;
    async fn status(&self, id: TxId) -> Result<BridgeStatus>;
}
```

* **`RainbowBridgeImpl`** – current light‑client logic (4–8 h, fully trustless).  
* **`OmniBridgeImpl`**     – MPC + Chain Signatures (minutes, cheaper gas).  

Load the implementation by `bridge.strategy` (env‐config: `rainbow`, `omni`, or `auto`).

---

## 3. Integration Plan Changes

| Phase | Current task | Updated task (if Omni added) |
|-------|--------------|------------------------------|
| **4.2.x – NEAR Adapter** | Implement Rainbow light‑client sync | Integrate **Omni SDK** & verify MPC signature |
| **4.3.1 – BridgeService** | Hard‑coded Rainbow logic | Instantiate `IBridge` from config ➜ plug `OmniBridgeImpl` |
| **4.3.7 – Time‑outs** | Handle 4 h challenge rollback | Parameterise by bridge type (short TTL for Omni) |
| **7.4.6 – Front‑end UX** | Progress bar for 4–8 h flow | Real‑time status + “Switch to Trustless mode” button |
| **5.1.3 – Risk Engine** | Monitor block proof delays | Add metrics: `mpc_node_latency`, `signature_age` |

**Wallet integrations remain unchanged** – RainbowKit / Wallet Selector keep signing transactions as before.

---

## 4. Roll‑out Strategy

| Stage | Bridge mode | Suggested limits | Goal |
|-------|-------------|------------------|------|
| **Hackathon / MVP** (2‑3 wk) | • Rainbow = default  <br>• Omni = experimental toggle | Omni for amounts \< $1 k | Showcase instant UX **and** trustless baseline |
| **Pilot (Mainnet Q4 2025)** | Dual‑mode | Omni for \< $5 k • Rainbow for \> $5 k | Gather real‑world stats, user preference |
| **Production 2026** | Tune `auto` strategy by SLA • cost • risk | TBD | Optimise latency vs. security automatically |

---

## 5. Risk Mitigation & Monitoring

1. **MPC Trust Model** – Require ≥ ⅔ honest Omni signers; audit validator set quarterly.  
2. **Fallback Logic** – If `status()` exceeds SLAs, auto‑switch to Rainbow & notify user.  
3. **Metrics & Alerts** – Export `bridge_latency`, `mpc_node_latency`, `gas_spent_eth`, `error_rate` to Prometheus + Grafana dashboards.  
4. **Security Audits** – Maintain separate audit trails for Rainbow (light‑client proofs) and Omni (MPC key‑ceremony & signature code).  

---

## 6. Next Steps Checklist

- [ ] Add `IBridge` trait & implementations skeletons.  
- [ ] Integrate **Omni Bridge SDK** in dev‑net; benchmark latency & gas.  
- [ ] Extend BridgeService API: `strategy` param (`auto` / `rainbow` / `omni`).  
- [ ] Update DB schema – store `bridge_type` per transaction.  
- [ ] Front‑end: progress component with ETA, “switch mode” control, SLA tool‑tips.  
- [ ] Sentry / Grafana alerts for new metrics.  

---

**Contact**: `bridge-team@kembridge.dev` for implementation questions.  

---
