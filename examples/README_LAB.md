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

# Next Lab

## Lab 5 — Uncertainty, Confidence, Calibration and Abstention

คำถามหลัก:

> ระบบรู้ได้อย่างไรว่าควรเชื่อ prediction ของตัวเองแค่ไหน?

และ:

> เมื่อไม่มั่นใจ ควรเลือกเอง หรือส่งต่อให้ระบบที่คิดละเอียดกว่า / Human?

แนวทางจะต่อจาก failure ที่พบใน Lab 4B+:

```text
Context = 0.40

Predicted:
LLM = 0.588
RAG = 0.564
SQL = 0.538

Model chooses:
LLM

But true best:
SQL
```

Lab 5 จะเริ่มศึกษาความแตกต่างระหว่าง:

```text
Prediction
Confidence
Calibration
Abstain
```

เพื่อพัฒนาจาก:

```text
"เลือก Action ที่คะแนนสูงสุด"
```

ไปสู่:

```text
"เลือกเมื่อมีหลักฐานเพียงพอ
และรู้ว่าเมื่อไรไม่ควรเดา"
```
