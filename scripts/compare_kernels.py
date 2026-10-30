import numpy as np
import scipy.special as sp

mu = 2.6923e10
nu = 0.3
rho = 2500.0
lambda_ = 2.0 * mu * nu / (1.0 - 2.0 * nu) # or plane stress

cp = np.sqrt((lambda_ + 2.0 * mu) / rho)
cs = np.sqrt(mu / rho)

r = 1.0 # 1 meter
f = 3.0 # 3 Hz
omega = 2.0 * np.pi * f
s = 0.02 * omega + 1j * omega

kp = s / cp
ks = s / cs
zp = kp * r
zs = ks * r

k0_s = sp.k0(zs)
k1_s = sp.k1(zs)
k1_p = sp.k1(zp)
k2_s = sp.kn(2, zs)
k2_p = sp.kn(2, zp)

gamma = cs / cp
gamma_sq = gamma**2

chi = k2_s - k2_p * gamma_sq
psi = k0_s + (k1_s - k1_p * gamma) / zs

prefactor = 1.0 / (2.0 * np.pi * rho * cs**2)

# Dynamic U_xx at dr = [1, 0]
u_dyn = (psi - chi) * prefactor

# Static Kelvin U_xx at dr = [1, 0]
c1 = 1.0 / (8.0 * np.pi * mu * (1.0 - nu))
kappa = 3.0 - 4.0 * nu
u_stat = c1 * (kappa * np.log(1.0 / r) + 1.0)

print(f"cp = {cp}, cs = {cs}")
print(f"|zp| = {np.abs(zp)}, |zs| = {np.abs(zs)}")
print(f"Dynamic U_xx = {u_dyn}")
print(f"Static  U_xx = {u_stat}")
print(f"Ratio Dynamic/Static = {u_dyn / u_stat}")
