import math

rho = 7850.0
e0 = 2.1e11
nu = 0.30
mu = e0 / (2.0 * (1.0 + nu))
lam = e0 * nu / ((1.0 + nu) * (1.0 - 2.0 * nu))
cp = math.sqrt((lam + 2.0 * mu) / rho)
cs = math.sqrt(mu / rho)
gamma = cs / cp

s = 1e-3
kp = s / cp
ks = s / cs

prefactor = 1.0 / (2.0 * math.pi * rho * cs * cs)
c1 = 1.0 / (8.0 * math.pi * mu * (1.0 - nu))
kappa = 3.0 - 4.0 * nu

euler_gamma = 0.5772156649015328606
# K0(z) ~ -ln(z/2) - euler_gamma
# When zs -> 0:
# psi ~ -ln(ks*r/2) - euler_gamma + 1/(4*(1-nu)) * (-ln(ks*r/2) - euler_gamma - gamma^2*ln(gamma)) ...
# Let's compute the constant term:
const_term = -math.log(ks / 2.0) - euler_gamma
print("ks =", ks)
print("const_term =", const_term)
print("prefactor =", prefactor)
print("Shift in U* =", prefactor * const_term)
