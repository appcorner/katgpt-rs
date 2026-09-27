# katgpt-rs Learning Labs

ชุดการทดลองนี้จัดทำขึ้นเพื่อศึกษาแนวคิดจาก `katgpt-rs` โดยเน้นการนำไปประยุกต์กับ **AI Worker / Agent / Harness / Decision System** มากกว่าการใช้งานด้านเกม

เป้าหมายของ Labs ไม่ใช่เพียงทำให้ example รันได้ แต่ต้องการตอบคำถามทีละขั้นว่า:

```text
AI มีหลายทางเลือก
        ↓
ควรเลือกอะไร?
        ↓
อะไรได้รับอนุญาตให้เลือก?
        ↓
สถานการณ์ปัจจุบันมีผลต่อการเลือกหรือไม่?
        ↓
ถ้าเจอสถานการณ์ใหม่ที่ไม่เคยเห็น จะทำอย่างไร?
```

## Catalog

| Lab | ไฟล์ | คำสั่งรัน | หัวข้อ |
| --- | --- | --- | --- |
| Lab 1 | [`bandit_01_basic.rs`](bandit_01_basic.rs) | `cargo run --example bandit_01_basic` | Multi-armed bandit; ต้นแบบจาก example นี้ |
| Lab 2 | [`bandit_01_basic.rs`](bandit_01_basic.rs) | `cargo run --example bandit_01_basic` | Constraint / Pruner; ต้นแบบจาก example เดียวกับ Lab 1 |
| Lab 3 | [`lab3_context_baseline.rs`](lab3_context_baseline.rs) | `cargo run --example lab3_context_baseline` | Context-aware bandit baseline |
| Lab 4 | [`lab4_generalization.rs`](lab4_generalization.rs) | `cargo run --example lab4_generalization` | Generalization ไปยัง context ที่ไม่เคยเห็น |
| Lab 5A | [`lab5_uncertainty.rs`](lab5_uncertainty.rs) | `cargo run --example lab5_uncertainty` | Decision margin และ ABSTAIN เมื่อคะแนนอันดับต้นใกล้กัน |
| Lab 5B | [`lab5b_calibration.rs`](lab5b_calibration.rs) | `cargo run --example lab5b_calibration` | วัด calibration ของ naive confidence ที่สร้างจาก decision margin |
| Lab 5C | [`lab5c_calibrated_confidence.rs`](lab5c_calibrated_confidence.rs) | `cargo run --example lab5c_calibrated_confidence` | เรียนรู้ histogram calibration แล้วประเมิน mapping บน test set อิสระ |
| Lab 5D | [`lab5d_calibrated_abstention.rs`](lab5d_calibrated_abstention.rs) | `cargo run --example lab5d_calibrated_abstention` | เปรียบเทียบ margin-based กับ calibrated-confidence abstention |

Labs ปัจจุบัน:

| Lab   | Topic               | คำถามหลัก                                |
| ----- | ------------------- | ---------------------------------------- |
| Lab 1 | Bandit              | อะไรให้ผลดีที่สุด?                       |
| Lab 2 | Constraint / Pruner | อะไรมีสิทธิ์ถูกเลือก?                    |
| Lab 3 | Context             | อะไรดีที่สุดสำหรับสถานการณ์นี้?          |
| Lab 4 | Generalization      | ถ้าเจอสถานการณ์ใหม่ จะประมาณคำตอบได้ไหม? |

---

# Lab 1 — Multi-Armed Bandit

## ปัญหา

สมมติระบบมีหลายทางเลือก แต่ยังไม่รู้ว่าทางเลือกใดให้ผลดีที่สุด

```text
Arm 0
Arm 1
Arm 2
Arm 3
Arm 4
```

ระบบต้องทดลอง เลือก action รับผลลัพธ์ แล้วเรียนรู้จากผลที่เกิดขึ้น

แนวคิดหลักคือ:

```text
Bandit
   ↓
เลือก Arm
   ↓
Environment
   ↓
Reward
   ↓
Update
   └────→ กลับไปเลือกใหม่
```

## Environment

ใช้ Bernoulli environment:

```text
Arm 0 → p = 0.2
Arm 1 → p = 0.5
Arm 2 → p = 0.8   ⭐ optimal
Arm 3 → p = 0.4
Arm 4 → p = 0.6
```

`p` คือโอกาสที่ Environment จะคืน:

```text
Reward = 1
```

ไม่ใช่ข้อมูลที่ Bandit รู้ล่วงหน้า

ตัวอย่าง:

```text
Bandit chooses Arm 2

random < 0.8
    ↓
Reward = 1

random >= 0.8
    ↓
Reward = 0
```

Bandit เห็นเพียง action ที่เลือกและ reward ที่ได้รับ

---

## Q-value

Q-value ใน Lab นี้หมายถึง:

> จากประสบการณ์ที่ผ่านมา ระบบประเมินว่าเลือก Arm นี้แล้วจะได้ผลดีประมาณเท่าไร

ตัวอย่าง:

```text
Arm 2
Q = 0.78
```

ไม่ได้หมายความว่า Arm 2 ถูกเลือก 78% ของเวลา

มันคือค่าประเมิน reward จากประสบการณ์ที่ผ่านมา

---

## Exploration vs Exploitation

Bandit ต้องจัดการสองเรื่องพร้อมกัน

### Exploration

ทดลองสิ่งที่ยังไม่แน่ใจ

### Exploitation

ใช้สิ่งที่ปัจจุบันเชื่อว่าดีที่สุด

ถ้า Exploit อย่างเดียว อาจติดอยู่กับตัวเลือกที่ดูดีในช่วงแรก

ถ้า Explore มากเกินไป ก็เสียโอกาสจากการไม่ใช้ตัวเลือกที่เรียนรู้แล้วว่าดี

---

## Regret

Regret ใช้ตอบคำถามว่า:

> เราเสียโอกาสไปเท่าไรจากการที่ยังไม่รู้ว่าตัวเลือกใดดีที่สุด?

ดังนั้นโดยทั่วไปเราต้องการ:

```text
Reward ↑

Regret ↓
```

---

## Thompson Sampling Bug Investigation

ระหว่าง Lab พบพฤติกรรมผิดปกติของ Thompson Sampling

ตัวอย่าง seed หลายค่าให้ pattern คล้ายกัน:

```text
Arm 2 true p = 0.8
Arm 4 true p = 0.6

แต่ Arm 4 ถูกเลือกประมาณ 91–93%
```

ทั้งที่ Q-value ของ Arm 2 สูงกว่า

การตรวจ code พบ root cause ใน Beta sampler

implementation เดิมใช้ Jöhnk rejection sampling และจำกัดจำนวนครั้งไว้ที่ 256

เมื่อ posterior มีขนาดใหญ่ sampling มีโอกาส fail สูงมาก แล้ว fallback เป็น:

```rust
0.5
```

เมื่อหลาย Arms ได้:

```text
sample = 0.5
```

selection ใช้:

```rust
if s >= best_score
```

ทำให้ Arm index หลังสุดชนะ tie

จึงเกิด feedback loop:

```text
Beta sampling fails
        ↓
sample = 0.5
        ↓
tie
        ↓
latest Arm wins
        ↓
Arm 4 selected
        ↓
posterior ใหญ่ขึ้น
        ↓
sampling fail บ่อยขึ้น
        ↓
Arm 4 selected ซ้ำ
```

### Fix

เปลี่ยน Beta sampling เป็นวิธีที่ stable กว่า โดยใช้ Gamma-ratio approach

หลังแก้:

| Seed | Arm2 Before | Arm2 After | Regret Before | Regret After |
| ---: | ----------: | ---------: | ------------: | -----------: |
|   42 |          24 |        863 |        205.60 |        35.90 |
|   43 |          21 |        936 |        208.00 |        23.10 |
|   44 |          28 |        918 |        204.90 |        21.90 |
|   45 |          42 |        963 |        202.40 |        11.70 |
|   46 |          29 |        881 |        203.70 |        32.70 |
|  999 |          46 |        950 |        200.20 |        18.30 |

พฤติกรรมกลับมาเป็นไปตามที่คาดหวัง

### บทเรียนสำคัญ

```text
Algorithm ถูก
≠
Implementation ถูก
```

และ:

```text
"Best arm" อย่างเดียว
ไม่เพียงพอสำหรับตรวจ behavior
```

ควรดูร่วมกับ:

```text
Visits
Reward
Regret
Q-values
```

---

# Lab 2 — Constraint / Pruner

## ปัญหา

Lab 1 ถามว่า:

> ตัวเลือกไหนดีที่สุด?

แต่ระบบจริงมีอีกคำถามที่ต้องตอบก่อน:

> ตัวเลือกไหนได้รับอนุญาตให้ใช้?

จึงเพิ่ม Pruner / Constraint ก่อน Bandit

```text
All Actions
     ↓
   Pruner
     ↓
Valid Actions
     ↓
   Bandit
     ↓
Best Valid Action
```

---

## Experiment B — Allow Arm 4

Arm 4 มี:

```text
true p = 0.9
```

และถูกอนุญาตให้เลือก

ผล:

```text
Arm | True p | Q-value | Visits
----|--------|---------|-------
0   | 0.1    | 0.0000  | 11
1   | 0.3    | 0.2353  | 17
2   | 0.7    | 0.6500  | 60
3   | 0.4    | 0.1875  | 16
4   | 0.9    | 0.9318  | 396 ⭐
```

Bandit ค้นพบ Arm 4 และเลือกมันมากที่สุด

---

## Experiment C — Block Arm 2 and Arm 4

กำหนด:

```text
Arm 2 p=.7 → BLOCKED
Arm 4 p=.9 → BLOCKED
```

ผล:

```text
Arm | True p | Q-value | Visits | Relevance | Status
----|--------|---------|--------|-----------|--------
0   | 0.1    | 0.0833  | 48     | 0.3948    |
1   | 0.3    | 0.2258  | 93     | 0.3943    |
2   | 0.7    | 0.0000  | 0      | 0.0000    | BLOCKED
3   | 0.4    | 0.4095  | 359    | 0.3970    | BEST
4   | 0.9    | 0.0000  | 0      | 0.0000    | BLOCKED
```

แม้ Arm 4 จะดีที่สุดในโลกทั้งหมด:

```text
Arm 4 = 0.9
```

Bandit ไม่มีสิทธิ์เลือกมัน

จึงได้:

```text
Best overall action = Arm 4

แต่

Best valid action = Arm 3
```

---

## Mental Model

```text
Pruner
"อะไรทำได้?"
     ↓
Valid Actions
     ↓
Bandit
"ในสิ่งที่ทำได้ เลือกอะไรดี?"
     ↓
Action
     ↓
Reward
     ↓
Learn
```

สรุป:

```text
Pruner = CAN I?

Bandit = WHICH ONE?

Reward = HOW DID IT GO?
```

---

# Lab 3 — Context

## ปัญหา

Lab 1 สมมติว่า Arm ที่ดีที่สุดเหมือนเดิมทุกสถานการณ์

แต่โลกจริงอาจเป็น:

```text
                  Knowledge    Customer Data

LLM                  0.9           0.3
RAG                  0.6           0.6
SQL                  0.2           0.9
```

ดังนั้นไม่มีคำตอบเดียวสำหรับคำถาม:

> Arm ไหนดีที่สุด?

ต้องถามว่า:

> Arm ไหนดีที่สุดสำหรับสถานการณ์นี้?

นี่คือแนวคิดของ **Context**

---

# Lab 3A — No-Context Baseline

Environment รู้ context แต่ Bandit ไม่รู้

```text
Knowledge ─┐
Customer  ─┤
Knowledge ─┤
Customer  ─┘
            ↓
          Bandit
```

Bandit จึงรวมประสบการณ์ทุก context เข้าด้วยกัน

ผล:

```text
Global Bandit Statistics

Arm          Q-value  Visits
LLM          0.5697   1097
RAG          0.5865   3722
SQL          0.5359   181
```

Best global arm:

```text
RAG
```

Reward:

```text
Total reward   = 2905
Average reward = 0.5810
```

ทั้งที่ RAG ไม่ใช่ action ที่ดีที่สุดใน context ใดเลย

ความจริงคือ:

```text
Knowledge → LLM = 0.9

Customer → SQL = 0.9
```

แต่ Bandit มองไม่เห็นข้อมูลนี้

### บทเรียน

```text
No Context
     ↓
รวมประสบการณ์ทุกสถานการณ์
     ↓
เรียนค่าเฉลี่ย
     ↓
Global Best
```

ระบบอาจเลือกตัวเลือกที่:

> ดีพอโดยเฉลี่ย

แทน:

> ดีที่สุดสำหรับสถานการณ์ปัจจุบัน

---

# Lab 3B — Context-Aware Baseline

รอบนี้ Bandit เห็น context

ใช้ `BanditStats` แยกชุด:

```text
Knowledge → BanditStats A

Customer → BanditStats B
```

ผล Knowledge:

```text
Arm          Q-value  Visits
LLM          0.8950   2418 ⭐
RAG          0.6667   27
SQL          0.4000   10
```

ผล Customer:

```text
Arm          Q-value  Visits
LLM          0.3333   6
RAG          0.4444   9
SQL          0.8984   2530 ⭐
```

ระบบเรียนรู้:

```text
Knowledge → LLM

Customer → SQL
```

Reward เพิ่มจาก:

```text
Lab 3A = 0.5810
```

เป็น:

```text
Lab 3B = 0.8930
```

---

## บทเรียนจาก Lab 3

Bandit:

```text
"อะไรดีที่สุด?"
```

Context-aware decision:

```text
"อะไรดีที่สุดสำหรับสถานการณ์นี้?"
```

อย่างไรก็ตาม Lab 3B ยังเป็นการแยก statistics เป็นกล่อง

```text
Context A → Stats A
Context B → Stats B
```

ถ้าเกิด Context ใหม่ที่ไม่เคยมีมาก่อน ระบบยังไม่มี statistics สำหรับมัน

นี่นำไปสู่ Lab 4

---

# Lab 4 — Generalization

## ปัญหา

สมมติ Context ไม่ได้มีเพียง:

```text
Knowledge
Customer
```

แต่เป็นค่าต่อเนื่อง:

```text
knowledge_score
```

กำหนด:

```text
0.0 = Customer-data task

1.0 = Knowledge task
```

ระหว่างกลางอาจเป็น:

```text
0.25
0.40
0.50
0.60
0.75
...
```

Environment:

```text
LLM:
p = 0.3 + 0.6 * knowledge_score

RAG:
p = 0.6

SQL:
p = 0.9 - 0.7 * knowledge_score
```

ดังนั้น:

```text
knowledge_score ↑

LLM reward ↑
RAG reward ─
SQL reward ↓
```

---

# Lab 4A — Memorization

Training contexts:

```text
0.00
0.25
0.75
1.00
```

จงใจไม่ train:

```text
0.50
```

ผล:

```text
Context  LLM      RAG      SQL
0.00     0.3244   0.5766   0.9001
0.25     0.4957   0.5399   0.7174
0.75     0.7480   0.5785   0.3516
1.00     0.8936   0.6225   0.2134
```

เมื่อถาม:

```text
Context = 0.50
```

ระบบตอบ:

```text
No learned statistics for this exact context.
```

เพราะมันทำงานเหมือน:

```text
0.00 → Stats A
0.25 → Stats B
0.75 → Stats C
1.00 → Stats D

0.50 → ???
```

นี่คือ **Memorization**

---

# Lab 4B — Generalizing Context Model

เปลี่ยนจากการจำแต่ละ context เป็นการเรียนความสัมพันธ์

Context ถูก represent เป็น:

```text
[knowledge_score, 1.0]
```

ตัวอย่าง:

```text
Context 0.75

φ = [0.75, 1.0]
```

ฝึกเฉพาะ:

```text
0.00
0.25
0.75
1.00
```

แต่สามารถประเมิน:

```text
0.50
```

ได้

ผล:

```text
Context  True LLM Pred LLM True RAG Pred RAG True SQL Pred SQL Best
0.00     0.30     0.361    0.60     0.539    0.90     0.817    SQL
0.25     0.45     0.503    0.60     0.555    0.72     0.642    SQL
0.50     0.60     0.644    0.60     0.570    0.55     0.468    LLM
0.75     0.75     0.786    0.60     0.585    0.38     0.294    LLM
1.00     0.90     0.927    0.60     0.601    0.20     0.119    LLM
```

`0.50` ไม่เคยอยู่ใน training data

แต่ model ยังสามารถประมาณผลได้

นี่คือ **Generalization**

---

# Lab 4B+ — Generalization Sweep

เพื่อพิสูจน์ว่า model ไม่ได้บังเอิญตอบ `0.50` ได้ จึงเพิ่ม unseen contexts:

```text
0.10
0.40
0.50
0.60
0.90
```

Training contexts ยังคงมีเพียง:

```text
0.00
0.25
0.75
1.00
```

ไม่มีการ retrain ด้วย evaluation contexts

---

## Results

```text
Context | Seen?  | True LLM | Pred LLM | True RAG | Pred RAG | True SQL | Pred SQL | Pred Best | True Best
0.00    | TRAIN  | 0.30     | 0.361    | 0.60     | 0.539    | 0.90     | 0.817    | SQL       | SQL
0.10    | UNSEEN | 0.36     | 0.418    | 0.60     | 0.545    | 0.83     | 0.747    | SQL       | SQL
0.25    | TRAIN  | 0.45     | 0.503    | 0.60     | 0.555    | 0.72     | 0.642    | SQL       | SQL
0.40    | UNSEEN | 0.54     | 0.588    | 0.60     | 0.564    | 0.62     | 0.538    | LLM       | SQL
0.50    | UNSEEN | 0.60     | 0.644    | 0.60     | 0.570    | 0.55     | 0.468    | LLM       | LLM
0.60    | UNSEEN | 0.66     | 0.701    | 0.60     | 0.576    | 0.48     | 0.398    | LLM       | LLM
0.75    | TRAIN  | 0.75     | 0.786    | 0.60     | 0.585    | 0.38     | 0.294    | LLM       | LLM
0.90    | UNSEEN | 0.84     | 0.871    | 0.60     | 0.595    | 0.27     | 0.189    | LLM       | LLM
1.00    | TRAIN  | 0.90     | 0.927    | 0.60     | 0.601    | 0.20     | 0.119    | LLM       | LLM
```

---

## Prediction Error

All contexts:

```text
MAE:
LLM     = 0.04428
RAG     = 0.03011
SQL     = 0.08212
Overall = 0.05217

MSE = 0.00338

Best-action matches = 8/9
```

TRAIN contexts:

```text
Overall MAE = 0.05225
MSE         = 0.00346
Best action = 4/4
```

UNSEEN contexts:

```text
Overall MAE = 0.05210
MSE         = 0.00331
Best action = 4/5
```

สิ่งที่สำคัญคือ:

```text
TRAIN error ≈ UNSEEN error
```

จึงมีหลักฐานว่า model ไม่ได้เพียงจำ training contexts

---

## Learned Trends

ผล prediction แสดงว่า:

```text
knowledge_score ↑

LLM
0.361 → 0.927
      ↗

RAG
0.539 → 0.601
      ─

SQL
0.817 → 0.119
      ↘
```

ตรงกับ structure ของ Environment

ดังนั้น model ได้เรียนรู้:

```text
Knowledge มากขึ้น
    ↓
LLM เหมาะขึ้น

Knowledge มากขึ้น
    ↓
SQL เหมาะน้อยลง

RAG
    ↓
ค่อนข้างคงที่
```

---

## Interesting Failure — Context 0.40

ที่:

```text
knowledge_score = 0.40
```

ค่าจริง:

```text
LLM = 0.54
RAG = 0.60
SQL = 0.62 ⭐
```

prediction:

```text
LLM = 0.588 ⭐
RAG = 0.564
SQL = 0.538
```

model เลือก LLM แต่ SQL คือ true best action

จุดนี้แสดงว่า:

```text
Generalization
≠
Prediction ถูกทุกครั้ง
```

โดยเฉพาะบริเวณที่หลาย Actions มี reward ใกล้กัน

นี่เป็นแรงจูงใจสำหรับการศึกษาต่อเรื่อง:

```text
Uncertainty
Confidence
Calibration
Abstention
```

---

# Context Vector และ Learned Weights

หลัง Lab 4 สามารถตีความศัพท์ที่ใช้ใน implementation ได้ง่ายขึ้น

## φ — Context / Feature Vector

`φ` คือ:

> ตัวเลขที่ใช้อธิบายสถานการณ์ปัจจุบัน

ตัวอย่าง:

```text
knowledge_score = 0.40

φ = [0.40, 1.0]
```

คิดง่าย ๆ ว่า:

```text
φ = "ตอนนี้สถานการณ์เป็นอย่างไร?"
```

---

## θ — Learned Weights

แต่ละ Action มีสิ่งที่ model เรียนรู้เกี่ยวกับความสัมพันธ์ระหว่าง context และ reward

```text
θ_LLM
θ_RAG
θ_SQL
```

คิดง่าย ๆ ว่า:

```text
θ = "จากประสบการณ์ที่ผ่านมา
     Action นี้เหมาะกับสถานการณ์แบบไหน?"
```

การตัดสินใจจึงมีภาพประมาณ:

```text
              Context
                 φ
                 │
        ┌────────┼────────┐
        ▼        ▼        ▼
      θ_LLM    θ_RAG    θ_SQL
        │        │        │
        ▼        ▼        ▼
      Score    Score     Score
        │        │        │
        └────────┼────────┘
                 ▼
              Action
```

---

# Overall Mental Model

หลังจาก Lab 1–4 เราได้ architecture ต่อเนื่องดังนี้:

```text
                 Context
                    │
                    ▼
              ┌──────────┐
              │  Pruner  │
              │ CAN I?   │
              └────┬─────┘
                   │
             Valid Actions
                   │
                   ▼
          ┌────────────────┐
          │ Decision Model │
          │ WHICH ONE?     │
          └───────┬────────┘
                  │
                Action
                  │
                  ▼
             Environment
                  │
                  ▼
                Reward
                  │
                  ▼
                Learn
```

เมื่อเพิ่ม Context และ Generalization:

```text
Context
   │
   ▼
Feature Vector φ
   │
   ▼
Learned Model θ
   │
   ▼
Action Scores
   │
   ▼
Best Valid Action
```

---

# Mapping to AI Worker / Harness

ตัวอย่างใน Labs สามารถเปลี่ยนจาก:

```text
LLM
RAG
SQL
```

เป็น strategies จริงของ AI Worker เช่น:

```text
Direct LLM
RAG
SQL Agent
Web Search
Tool Call
Human Escalation
```

Context อาจประกอบด้วย:

```text
task type
confidence
data availability
user permissions
risk
cost budget
latency budget
previous failures
```

Pruner สามารถจัดการ:

```text
permissions
policy
safety
tool availability
budget
```

Bandit / Decision Model จัดการ:

```text
ใน actions ที่ได้รับอนุญาต
strategy ไหนเหมาะกับสถานการณ์นี้?
```

Reward อาจมาจาก:

```text
task success
correctness
user acceptance
human intervention
cost
latency
safety violations
```

ดังนั้นแนวคิดจาก Labs ไม่จำเป็นต้องผูกกับเกม

มันสามารถกลายเป็น:

```text
AI Worker
   ↓
Observe Context
   ↓
Check Constraints
   ↓
Choose Strategy
   ↓
Execute
   ↓
Observe Outcome
   ↓
Reward
   ↓
Learn
```

---

# Lessons Learned So Far

## 1. Reward determines what the system learns

Bandit ไม่รู้ว่าอะไรคือสิ่งที่มนุษย์ต้องการ

มันเรียนจาก Reward ที่เราออกแบบ

ดังนั้น:

```text
Bad Reward Design
        ↓
Bad Learned Behavior
```

แม้ algorithm จะทำงานถูกต้องก็ตาม

---

## 2. Best overall is not always Best valid

Lab 2 แสดงว่า:

```text
Best overall action
≠
Best allowed action
```

Constraint ต้องอยู่ก่อน optimization

---

## 3. Global average can hide important structure

Lab 3 แสดงว่า:

```text
Global Best
```

อาจไม่มีความหมายมากนักเมื่อ Environment มีหลาย Context

---

## 4. Memorization is not Generalization

```text
Memorization:
"ฉันเคยเห็นสถานการณ์นี้"

Generalization:
"ฉันไม่เคยเห็นสถานการณ์นี้
แต่เข้าใจลักษณะของมัน"
```

---

## 5. Generalization is not certainty

Lab 4B+ แสดงว่า model สามารถจับ pattern ได้ แต่ยังเลือกผิดได้

ดังนั้นขั้นต่อไปไม่ควรถามเพียง:

```text
"What should I choose?"
```

แต่ควรถามเพิ่ม:

```text
"How confident am I?"

และ

"Should I choose at all?"
```

---

# Lab 5 — Confidence, Calibration and Abstention

Lab 1–4 พาเรามาถึงจุดที่ระบบสามารถ:

```text
ดูสถานการณ์
    ↓
ประเมิน Action ต่าง ๆ
    ↓
เลือก Action ที่น่าจะดีที่สุด
```

แต่ยังมีคำถามสำคัญที่เหลืออยู่:

> ถ้า Model เลือก Action หนึ่งขึ้นมา เราควรเชื่อการตัดสินใจนั้นมากแค่ไหน?

และคำถามที่สำคัญยิ่งกว่า:

> ถ้า Model ไม่แน่ใจ ควรปล่อยให้มันทำงานเองหรือไม่?

Lab 5 จึงศึกษาสี่เรื่องต่อเนื่องกัน:

| Lab | Topic | คำถามหลัก |
|---|---|---|
| 5A | Decision Margin + Abstention | Action ที่ชนะ ชนะขาดแค่ไหน? |
| 5B | Measuring Calibration | ตัวเลข Confidence ที่เราสร้างขึ้น เชื่อเป็น % ได้จริงไหม? |
| 5C | Calibration Mapping | จะเรียนรู้ความหมายของ Confidence จากผลจริงได้อย่างไร? |
| 5D | Calibrated Abstention | จะใช้ Confidence ที่ calibrate แล้วควบคุมว่า AI ควรทำเองหรือหยุดเมื่อไร? |

---

# 5.0 คำศัพท์ที่ควรรู้ก่อน

Lab 5 มีศัพท์ใหม่หลายคำ แต่จริง ๆ แล้วแต่ละคำตอบคำถามคนละเรื่อง

## Predicted Score / Predicted Reward

ค่าที่ Model ประเมินให้แต่ละ Action

ตัวอย่าง:

```text
LLM = 0.588
RAG = 0.564
SQL = 0.538
```

แปลแบบง่าย:

> Model คิดว่า LLM ดูดีที่สุดในสถานการณ์นี้

แต่ `0.588` ยังไม่ได้หมายความว่า:

> LLM มีโอกาสถูก 58.8%

มันเป็นเพียงคะแนนที่ Model ใช้เปรียบเทียบ Actions

---

## Decision Margin

ความต่างระหว่าง Action อันดับหนึ่งกับอันดับสอง

ตัวอย่าง:

```text
LLM = 0.588
RAG = 0.564

Margin = 0.588 - 0.564
       = 0.024
```

แปลแบบง่าย:

> LLM ชนะ RAG นิดเดียว

ถ้า:

```text
LLM = 0.871
RAG = 0.595

Margin = 0.276
```

แปลว่า:

> LLM ชนะขาดกว่ามาก

ดังนั้น Margin ใช้เป็นสัญญาณง่าย ๆ ว่า Decision นั้น “สูสี” หรือ “ชัดเจน”

แต่:

```text
Margin ≠ Probability
```

`Margin = 0.20` ไม่ได้แปลว่า “มั่นใจ 20%”

---

## Confidence

ในความหมายทั่วไปคือ:

> ระบบคิดว่าการตัดสินใจของตัวเองน่าเชื่อถือแค่ไหน

แต่คำว่า Confidence ต้องใช้ระวังมาก

ถ้าเราเอา Margin มาคูณเลขบางตัวแล้วได้:

```text
0.72
```

ไม่ได้แปลว่าเรามี “72% Confidence” ที่มีความหมายทางสถิติทันที

---

## Calibration

Calibration แปลแบบง่าย ๆ ว่า:

> ตรวจว่าตัวเลข Confidence ที่ระบบพูดออกมา สอดคล้องกับความถูกต้องจริงหรือไม่

ตัวอย่าง:

ถ้าระบบพูดว่า:

```text
Confidence ≈ 80%
```

จำนวน 1,000 ครั้ง

ถ้า calibrated ดี เราอยากเห็นว่าระบบถูกประมาณ:

```text
~800 ครั้ง
```

ถ้ามันพูดว่า 80% แต่ถูกแค่ 50%:

```text
Overconfident
```

แปลว่า:

> มั่นใจเกินจริง

ถ้าพูดว่า 40% แต่จริง ๆ ถูก 80%:

```text
Underconfident
```

แปลว่า:

> มั่นใจต่ำกว่าความสามารถจริง

---

## Abstain

`ABSTAIN` หมายถึง:

> ระบบมี Prediction แต่เลือกที่จะไม่ลงมือทำเอง

ไม่ใช่ Error

ไม่ใช่ Failure

แต่เป็น Decision หนึ่งว่า:

```text
"ข้อมูลยังไม่พอให้ฉันทำเอง"
```

ในระบบจริง ABSTAIN อาจหมายถึง:

```text
ส่งต่อให้ LLM ที่เก่งกว่า
ใช้ RAG/Search เพิ่ม
เรียก Tool เพิ่ม
ถาม Human
หรือเข้าสู่ System 2
```

---

## Coverage

Coverage คือ:

> ระบบยอมทำงานเองกี่เปอร์เซ็นต์

เช่น:

```text
100 งาน
AI ทำเอง 70 งาน
ส่งต่อ 30 งาน

Coverage = 70%
```

---

## Selective Accuracy

Selective Accuracy คือ:

> เฉพาะงานที่ AI ยอมทำเอง มันทำถูกกี่เปอร์เซ็นต์

ตัวอย่าง:

```text
AI ทำเอง 70 งาน
ถูก 69 งาน

Selective Accuracy = 98.6%
```

ดังนั้น Coverage และ Selective Accuracy ต้องดูคู่กัน

เพราะ:

```text
ทำเอง 1 งาน
ถูก 1 งาน
```

ก็ได้ Accuracy 100%

แต่แทบไม่มี Productivity

---

# Lab 5A — Decision Margin and Abstention

## ปัญหา

จาก Lab 4 เราพบสถานการณ์:

```text
Context = 0.40

Predicted:
LLM = 0.588
RAG = 0.564
SQL = 0.538
```

Model เลือก:

```text
LLM
```

แต่ค่าจริงคือ:

```text
LLM = 0.54
RAG = 0.60
SQL = 0.62
```

ดังนั้น True Best คือ:

```text
SQL
```

Model เลือกผิด

แต่ถ้ามองเพิ่ม:

```text
LLM = 0.588
RAG = 0.564

Margin = 0.024
```

จะเห็นว่าอันดับหนึ่งกับอันดับสองสูสีกันมาก

จึงเกิดแนวคิด:

```text
ถ้า Margin สูง
→ EXECUTE

ถ้า Margin ต่ำ
→ ABSTAIN
```

---

## Experiment

กำหนด:

```text
ABSTAIN_THRESHOLD = 0.10
```

Policy:

```text
margin >= 0.10
    → EXECUTE

margin < 0.10
    → ABSTAIN
```

---

## Result

ก่อน Abstention:

```text
Raw accuracy = 8/9 = 88.9%
```

หลังใช้ Threshold 0.10:

```text
EXECUTE = 6
ABSTAIN = 3

Correct EXECUTE = 6
Wrong EXECUTE = 0

Coverage = 66.7%
Selective Accuracy = 100%
```

สำคัญที่สุดคือ Wrong Prediction ที่ Context `0.40` ถูกจับไว้ในกลุ่ม:

```text
ABSTAIN
```

---

## Threshold Sweep

ผลจาก Threshold หลายค่า:

| Threshold | Coverage | Selective Accuracy |
|---:|---:|---:|
| 0.00 | 100% | 88.9% |
| 0.02 | 100% | 88.9% |
| 0.05 | 88.9% | 100% |
| 0.10 | 66.7% | 100% |
| 0.15 | 55.6% | 100% |
| 0.20 | 55.6% | 100% |
| 0.30 | 11.1% | 100% |

เห็น Trade-off ชัดเจน:

```text
Threshold ต่ำ
    ↓
AI ทำงานเองเยอะ
    ↓
Coverage สูง
    ↓
อาจปล่อย Wrong Decision มากขึ้น


Threshold สูง
    ↓
AI ระมัดระวังมากขึ้น
    ↓
Coverage ลดลง
    ↓
ส่งต่อมากขึ้น
```

---

## บทเรียนจาก Lab 5A

```text
Margin = สัญญาณว่า
"อันดับหนึ่งชนะอันดับสองขาดแค่ไหน"
```

Margin มีประโยชน์ในการสร้าง Abstention Policy

แต่:

```text
Margin ≠ Confidence Probability
```

ดังนั้นยังไม่ควรพูดว่า:

```text
margin = 0.20
→ Confidence = 20%
```

---

# Lab 5B — Measuring Calibration

## ปัญหา

เราต้องการรู้ว่า:

> ถ้าเราแปลง Margin ให้กลายเป็นตัวเลข 0–1 แล้วเรียกว่า Confidence เราเชื่อเลขนั้นได้หรือไม่?

เพื่อทดลอง เราจงใจสร้าง:

```text
naive_confidence
    = clamp(margin × 4.0, 0, 1)
```

เลข `4.0` เป็นค่าที่กำหนดขึ้นเอง

ไม่ได้เรียนจากข้อมูล

ไม่ได้มีความหมายทาง Probability

---

## Evaluation

ใช้:

```text
10,000 decisions
```

และเปรียบเทียบ:

```text
Naive Confidence
       vs
Decision ถูกจริงหรือไม่
```

จัด Confidence เป็นช่วง เช่น:

```text
0–10%
10–20%
20–30%
...
90–100%
```

แล้วดูว่าแต่ละกลุ่มถูกจริงกี่ %

---

## Result

ตัวอย่าง:

```text
Mean Confidence ≈ 5.7%
Actual Accuracy ≈ 38.4%
```

อีกกลุ่ม:

```text
Mean Confidence ≈ 35%
Actual Accuracy = 100%
```

เห็นชัดว่า:

```text
Naive Confidence
≠
Actual Probability of Correctness
```

---

## Overall Result

```text
Mean naive confidence = 0.593
Actual accuracy       = 0.858
ECE                   = 0.265
```

ระบบจึงมีลักษณะ:

```text
Underconfident
```

คือ:

> ตัวเลข Confidence ต่ำกว่าความถูกต้องจริงโดยรวม

---

## ECE คืออะไร?

`ECE` = Expected Calibration Error

ใช้สรุปว่า:

> ตัวเลข Confidence ที่รายงาน ห่างจาก Accuracy จริงมากแค่ไหน

คิดง่าย ๆ:

```text
Reported Confidence
         vs
Actual Accuracy
         ↓
ดูความต่างแต่ละกลุ่ม
         ↓
รวมออกมาเป็น ECE
```

โดยทั่วไป:

```text
ECE ต่ำลง
→ Calibration ดีขึ้น
```

แต่:

```text
ECE ต่ำ
≠
ระบบปลอดภัย
≠
Decision ถูกเสมอ
```

---

## Boundary Analysis

พบว่า Model มีปัญหามากบริเวณ:

```text
knowledge_score ≈ 0.40–0.60
```

ผล:

```text
Context Range    Accuracy

0.00–0.20        100%
0.20–0.40         79.9%
0.40–0.60         50.5%
0.60–0.80        100%
0.80–1.00        100%
```

ตรงกลางคือบริเวณที่ Actions แข่งขันกันมาก

---

## บทเรียนจาก Lab 5B

```text
Predicted Reward
    ≠
Decision Margin
    ≠
Probability Decision Is Correct
```

และ:

```text
ค่าที่อยู่ระหว่าง 0–1
ไม่ได้แปลว่า
เป็น Probability
```

หรือสั้น ๆ:

```text
bounded ≠ calibrated
```

---

# Lab 5C — Learning a Calibration Mapping

## ปัญหา

จาก Lab 5B เราพบว่า Naive Confidence ไม่ตรงกับ Accuracy จริง

คำถามต่อไปคือ:

> เราสามารถเรียนรู้ Mapping จาก Confidence เดิม ไปเป็น Probability ที่มีความหมายขึ้นได้ไหม?

---

## Data Separation

เพิ่มแนวคิดสำคัญมาก:

```text
Training Set
Calibration Set
Test Set
```

สามชุดมีหน้าที่ต่างกัน

### Training Set

ใช้ฝึก Decision Model

### Calibration Set

ใช้เรียนว่า:

```text
signal แบบนี้
→ ในอดีตถูกกี่ %
```

### Test Set

ใช้ตรวจสอบ Calibrator หลังจาก Freeze แล้ว

ห้ามใช้ Test Set กลับไปปรับ Calibrator

เพราะจะเท่ากับ:

> แอบดูข้อสอบก่อนสอบจริง

---

## Histogram Calibration

วิธีที่ใช้ใน Lab นี้ง่ายมาก

แบ่ง Naive Confidence เป็นช่วง:

```text
0.0–0.1
0.1–0.2
...
0.9–1.0
```

แล้วดูจาก Calibration Dataset ว่า:

```text
ในช่วงนี้
Decision ถูกจริงกี่ %
```

ตัวอย่าง:

```text
Naive confidence 0.0–0.1
Actual correctness ≈ 42.5%

ดังนั้น
Calibrated confidence ≈ 0.425
```

---

## Learned Mapping

ตัวอย่าง:

```text
Naive 0.0–0.1 → Calibrated ≈ 0.425
Naive 0.1–0.2 → Calibrated ≈ 0.370
Naive 0.2–0.3 → Calibrated ≈ 0.430

Naive >= 0.3
→ Calibrated ≈ 1.0
```

จากนั้น Freeze Mapping นี้

---

## Independent Test Result

ก่อน Calibration:

```text
Decision accuracy       = 0.8623
Mean confidence         = 0.6020
ECE                     = 0.2603
Maximum calibration gap = 0.6502
```

หลัง Calibration:

```text
Decision accuracy       = 0.8623
Mean confidence         = 0.8641
ECE                     = 0.0040
Maximum calibration gap = 0.0200
```

สิ่งสำคัญที่สุด:

```text
Decision Accuracy
Before = After
```

เพราะ Calibration ไม่ได้เปลี่ยน Action ที่ Model เลือก

มันเปลี่ยนเพียง:

> เราควรตีความความน่าเชื่อถือของ Decision นั้นอย่างไร

---

## Mental Model

```text
Decision Model
"เลือกอะไร?"
      ↓

Confidence Signal
"Decision นี้ดูชัดแค่ไหน?"
      ↓

Calibrator
"จากประสบการณ์จริง
signal แบบนี้ถูกบ่อยแค่ไหน?"
      ↓

Calibrated Probability
```

---

## Global vs Local Calibration

แม้ Overall ECE ดีมาก:

```text
0.004
```

แต่เมื่อแยก Context:

```text
Context       Calibration Gap

0.00–0.20        0.000
0.20–0.40        0.187
0.40–0.60        0.186
0.60–0.80        0.000
0.80–1.00        0.000
```

ตรงกลางยังไม่ดีนัก

ดังนั้น:

```text
Global Calibration ดี
≠
Local Calibration ดีทุกพื้นที่
```

---

## ข้อจำกัดของ Histogram Calibration

Histogram ทำให้ Confidence เหลือเพียงไม่กี่ระดับ

เช่น:

```text
~0.37
~0.42
~0.43
1.00
```

จึงเข้าใจง่าย แต่หยาบ

เหมาะกับการเรียน Concept

ยังไม่ใช่วิธีที่ควรสรุปว่าเหมาะที่สุดสำหรับ Production

---

# Lab 5D — Calibrated Confidence + Abstention

## เป้าหมาย

เอาสิ่งที่เรียนจาก:

```text
Lab 5A → Abstention

Lab 5C → Calibrated Probability
```

มารวมกัน

เราจึงมี Policy สองแบบ

---

## Policy A — Margin-based

```text
margin >= threshold
→ EXECUTE
```

ถามว่า:

> Action ที่ชนะ ชนะขาดเพียงพอหรือไม่?

---

## Policy B — Calibrated-confidence

```text
calibrated_confidence >= threshold
→ EXECUTE
```

ถามว่า:

> จากประสบการณ์ที่ผ่านมา Decision ที่มี signal แบบนี้ถูกบ่อยพอหรือไม่?

---

# Baseline

ถ้าทำทุก Decision:

```text
Total decisions = 20,000

Correct = 17,220
Wrong   = 2,780

Accuracy = 86.10%
Coverage = 100%
```

---

# Margin Policy Result

ตัวอย่าง:

```text
margin >= 0.10
```

ได้:

```text
Coverage = 68.36%
Correct Execute = 13,673
Wrong Execute = 0
Selective Accuracy = 100%
```

ตัวเลข `Wrong Execute = 0` หมายถึงไม่พบการตัดสินใจผิดในรายการที่ระบบเลือกทำ
ในการทดสอบชุดนี้เท่านั้น ไม่ได้แปลว่าความเสี่ยงจริงเป็นศูนย์หรือรับประกันว่า
การตัดสินใจครั้งต่อไปจะถูกต้องค่ะ

---

# Calibrated Policy Result

ถ้า:

```text
calibrated confidence >= 0.50
```

ได้:

```text
Coverage = 76.62%
Correct Execute = 15,325
Wrong Execute = 0
Selective Accuracy = 100%
```

เช่นเดียวกัน `Wrong Execute = 0` คือไม่พบการตัดสินใจผิดในรายการที่เลือกทำ
15,325 รายการของชุดทดสอบนี้ ไม่ใช่หลักประกันว่าความเสี่ยงในอนาคตเป็นศูนย์ค่ะ

ผลนี้เปรียบเทียบ Coverage 76.62% กับ 68.36% จึงไม่ได้ทดสอบที่ Coverage
เท่ากันค่ะ ในตัวอย่างมีส่วน `Similar-Coverage Comparison` สำหรับดูคู่ threshold
ที่มี Coverage ใกล้เคียงกัน ซึ่งช่วยให้เทียบสองนโยบายได้เป็นธรรมขึ้น

ใน Synthetic Experiment นี้ Calibrated Policy ทำ Coverage ได้มากกว่า Margin
Threshold ที่ให้ Wrong Execute = 0 แต่ผลนี้เป็นเพียงผลจากชุดทดลองนี้:

```text
ไม่ได้พิสูจน์ว่า
Calibrated Policy ดีกว่าเสมอ
```

---

# Risk–Coverage Trade-off

## No Abstention

```text
Coverage = 100%
Risk     = 13.90%
```

## Margin 0.10

```text
Coverage = 68.36%
Risk     = 0%
```

## Calibrated >= 0.50

```text
Coverage = 76.62%
Risk     = 0%
```

ใน Lab นี้เราเห็นแนวคิด:

```text
Autonomy สูง
     ↕
Reliability สูง
```

มักต้อง Trade-off กัน

---

# Context Region Analysis

เมื่อใช้:

```text
Calibrated Confidence >= 0.90
```

ได้:

```text
Context       Coverage

0.00–0.20       100%
0.20–0.40        34%
0.40–0.60        50%
0.60–0.80       100%
0.80–1.00       100%
```

ระบบจึง Abstain มากขึ้นบริเวณที่ Decision ยาก

และกล้าทำเองเต็มที่บริเวณที่ Action หนึ่งชัดเจนมาก

---

# ข้อจำกัดที่ค้นพบ

เพราะ Histogram Calibration หยาบ:

```text
threshold .50
threshold .60
threshold .70
threshold .80
threshold .90
threshold .95
threshold .99
```

ให้ Coverage เท่ากัน:

```text
76.62%
```

เพราะ Confidence ที่ Calibration สร้างขึ้นมีเพียงไม่กี่ระดับ

ดังนั้น Threshold หลายค่าไปตกอยู่ใน Policy เดียวกัน

นี่ไม่ใช่ Bug

แต่เป็นข้อจำกัดของ Calibration Method ที่ใช้

---

# Lab 5 Mental Model

ตอนนี้ Decision System มี flow:

```text
                  Context
                     │
                     ▼
               Decision Model
                     │
              Action Scores
                     │
               ┌─────┴─────┐
               ▼           ▼
             Top 1       Top 2
               │           │
               └─────┬─────┘
                     ▼
                   Margin
                     │
                     ▼
                 Calibrator
                     │
                     ▼
          Calibrated Confidence
                     │
              ┌──────┴──────┐
              │             │
         High enough       Low
              │             │
              ▼             ▼
           EXECUTE       ABSTAIN
                           │
                           ▼
                      System 2 /
                      Human /
                      More Tools
```

---

# Mapping to AI Worker / Harness

ในระบบจริง Action อาจเป็น:

```text
Direct LLM
RAG
SQL Agent
Search
Send Email
Create Document
Call API
Human Escalation
```

Decision Model เลือกว่า:

> Action ไหนเหมาะที่สุดใน Context นี้?

Calibration ช่วยตอบว่า:

> การตัดสินใจลักษณะนี้ ในอดีตน่าเชื่อถือเพียงใด?

Abstention Policy ตอบว่า:

> จากระดับความน่าเชื่อถือและ Risk ของงาน เราควรให้ AI ทำเองหรือไม่?

---

# ตัวอย่าง Low-risk / High-risk

## Low-risk

เช่น:

```text
สรุปบทความ
แนะนำเอกสาร
จัดหมวดหมู่ Ticket
```

ผิดแล้วผลกระทบน้อย

อาจใช้:

```text
Confidence Threshold ต่ำกว่า
→ Coverage สูง
```

---

## High-risk

เช่น:

```text
อนุมัติ Payment
แก้ Master Data
ส่ง Email ออกนอกองค์กร
ลบข้อมูล
```

ผิดแล้วผลกระทบสูง

อาจต้อง:

```text
Confidence Threshold สูง
หรือ
บังคับ Human Approval
```

ดังนั้นใน Production:

```text
Threshold เดียว
อาจไม่เหมาะกับทุก Action
```

---

# Key Lessons from Lab 5

## 1. Prediction ไม่เท่ากับ Confidence

```text
"ฉันเลือก LLM"
```

กับ:

```text
"ฉันน่าเชื่อถือแค่ไหนที่เลือก LLM"
```

เป็นคนละคำถาม

---

## 2. Margin เป็น Signal ไม่ใช่ Probability

```text
Margin = Top1 - Top2
```

ช่วยบอกว่าการแข่งขันสูสีแค่ไหน

แต่ไม่บอกตรง ๆ ว่า Decision ถูกกี่ %

---

## 3. Confidence ที่อยู่ในช่วง 0–1 ยังไม่ใช่ Probability

```text
0 <= confidence <= 1
```

ไม่เพียงพอ

ต้องตรวจด้วยข้อมูลจริง

---

## 4. Calibration ให้ความหมายเชิง Empirical

Calibration พยายามตอบ:

> Decision ที่มี signal แบบนี้ ในข้อมูลที่ผ่านมา ถูกจริงกี่เปอร์เซ็นต์?

---

## 5. Calibration ไม่ได้ทำให้ Model ฉลาดขึ้น

```text
Before Calibration
Accuracy = 86.23%

After Calibration
Accuracy = 86.23%
```

เหมือนเดิม

Calibration เปลี่ยน:

```text
ความหมายของ Confidence
```

ไม่ใช่:

```text
ความสามารถในการเลือก Action
```

---

## 6. AI ไม่จำเป็นต้องทำเองทุกครั้ง

AI Worker ที่ดีอาจไม่ใช่ระบบที่:

```text
Automation = 100%
```

แต่เป็นระบบที่:

```text
ทำเองเมื่อ Evidence เพียงพอ
และ
รู้ว่าเมื่อไรควรส่งต่อ
```

---

## 7. Coverage ต้องดูคู่กับ Accuracy

```text
Selective Accuracy = 100%
```

อาจไม่มีความหมายถ้า:

```text
Coverage = 1%
```

ดังนั้นต้องดู:

```text
Coverage
Selective Accuracy
Wrong Autonomous Actions
```

พร้อมกัน

---

## 8. Global Calibration อาจซ่อน Local Failure

Overall ECE ที่ดีมาก ไม่ได้แปลว่า Calibration ดีทุก Context

ต้องตรวจบริเวณสำคัญแยกด้วย

---

# Summary: Lab 1 → Lab 5

หลังจาก Lab 1–5 เราได้ Mental Model ต่อเนื่อง:

```text
Lab 1 — Bandit
"What tends to work best?"
        │
        ▼
Lab 2 — Constraint / Pruner
"What am I allowed to do?"
        │
        ▼
Lab 3 — Context
"What works best in this situation?"
        │
        ▼
Lab 4 — Generalization
"What about a situation I have never seen exactly?"
        │
        ▼
Lab 5 — Confidence / Calibration / Abstention
"How much should I trust this decision,
and should I act autonomously?"
```

หรือย่อเป็นภาษาไทย:

```text
เลือกอะไรดี?
    ↓
อะไรทำได้?
    ↓
สถานการณ์นี้ควรเลือกอะไร?
    ↓
สถานการณ์ใหม่จะประมาณได้ไหม?
    ↓
เชื่อการตัดสินใจนี้ได้แค่ไหน?
    ↓
ควรทำเอง หรือควรส่งต่อ?
```

---

# Next Direction

Lab ต่อไปควรศึกษาว่า:

> ถ้าความเสียหายของ Action แต่ละชนิดไม่เท่ากัน เราควรใช้ Confidence Threshold เดียวกันหรือไม่?

ตัวอย่าง:

```text
ตอบ FAQ ผิด
        ≠
ส่ง Email ผิด
        ≠
อนุมัติ Payment ผิด
        ≠
ลบข้อมูลผิด
```

จึงนำไปสู่หัวข้อถัดไป:

```text
Risk-aware Decision
Cost-aware Decision
Expected Utility
Action-specific Thresholds
```

ซึ่งจะทำให้ระบบขยับจาก:

```text
"เลือก Action ที่น่าจะดีที่สุด"
```

ไปสู่:

```text
"เลือก Action ที่ให้ประโยชน์เหมาะสม
เมื่อคำนึงถึง Reward, Cost, Risk
และ Confidence"
```
