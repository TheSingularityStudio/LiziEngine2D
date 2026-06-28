# LiziEngine2D - 2D Electrostatic CPU Simulator (Python)

## Step 1: Project scaffold
- [x] Create `requirements.txt`
- [ ] Create `pyproject.toml` (or keep simple with requirements only)
- [x] Create `src/` package and basic modules

## Step 2: Core simulation implementation
- [x] Implement grid + coordinate mapping
- [x] Implement particle state (positions, velocities)
- [x] Implement scatter: particles -> grid charge density (ρ) using bilinear weighting
- [x] Implement Poisson solver via discrete Green’s function kernel + **FFT-based cyclic convolution**
- [x] Implement gradient: E = -∇V from grid potentials
- [x] Implement gather: particle force from interpolated E (consistent with scatter interpolation)
- [x] Implement half-implicit Euler integrator

## Step 3: Validation / testing (critical-path)
- [x] `scripts/validate_single_charge.py` (single charge: symmetry/direction checks)
- [x] `scripts/validate_two_charges.py` (two charges: superposition/direction checks)
- [x] `scripts/validate_random.py` (random charges: numerical stability)

## Step 4: Run & report
- [ ] Execute the three validation scripts
- [ ] Adjust constants/discretization if errors are unacceptable
