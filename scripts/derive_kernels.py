# Let's derive T* directly from U* using Hooke's law:
# U_{ik} = 1/(2*pi*mu) * [ psi(r) delta_{ik} - chi(r) r_i r_k / r^2 ]
#
# Let's write the exact differentiation in terms of psi, chi and their derivatives.
# Let x be the field point, xi be the source point:
# r_i = x_i - xi_i
# dr/dx_j = r_j / r
# d(r_i)/dx_j = delta_{ij}
# d(dr_i)/dx_j = (delta_{ij} - dr_i dr_j) / r where dr_i = r_i / r
#
# U_{ik, j} = 1/(2*pi*mu) * [
#   psi' dr_j delta_{ik}
#   - chi' dr_j dr_i dr_k
#   - chi ( (delta_{ij} - dr_i dr_j)/r dr_k + dr_i (delta_{kj} - dr_k dr_j)/r )
# ]
# = 1/(2*pi*mu) * [
#   psi' dr_j delta_{ik}
#   - (chi' - 2*chi/r) dr_i dr_j dr_k
#   - (chi/r) (delta_{ij} dr_k + delta_{kj} dr_i)
# ]
#
# Now strain / divergence:
# U_{im, m} = 1/(2*pi*mu) * [
#   psi' dr_i
#   - (chi' - 2*chi/r) dr_i (sum dr_m^2 = 1)
#   - (chi/r) (dr_i + 2 dr_i)
# ]   (note: in 2D, sum delta_{mm} = 2, so delta_{im} dr_m = dr_i, and delta_{mm} dr_i = 2 dr_i)
# = 1/(2*pi*mu) * [ psi' - chi' + 2*chi/r - 3*chi/r ] dr_i
# = 1/(2*pi*mu) * [ psi' - chi' - chi/r ] dr_i
#
# Now Hooke's law:
# sigma_{ikj} = lambda U_{im, m} delta_{kj} + mu (U_{ik, j} + U_{ij, k})
#
# Then traction T_{ik} = sigma_{ikj} n_j:
# 1) lambda U_{im, m} delta_{kj} n_j = lambda U_{im, m} n_k
#    = lambda / (2*pi*mu) * [ psi' - chi' - chi/r ] dr_i n_k
#
# 2) mu U_{ik, j} n_j = 1/(2*pi) * [
#    psi' (dr . n) delta_{ik}
#    - (chi' - 2*chi/r) dr_i dr_k (dr . n)
#    - (chi/r) (n_i dr_k + delta_{ik} (dr . n))   Wait! delta_{kj} n_j dr_i = n_k dr_i!
#    Let's check: (delta_{ij} dr_k + delta_{kj} dr_i) n_j = n_i dr_k + n_k dr_i!
# ]
#
# 3) mu U_{ij, k} n_j:
#    U_{ij, k} = 1/(2*pi*mu) * [
#      psi' dr_k delta_{ij}
#      - (chi' - 2*chi/r) dr_i dr_j dr_k
#      - (chi/r) (delta_{ik} dr_j + delta_{jk} dr_i)
#    ]
#    Contract with n_j:
#    psi' dr_k n_i
#    - (chi' - 2*chi/r) dr_i dr_k (dr . n)
#    - (chi/r) (delta_{ik} (dr . n) + n_k dr_i)
#
# Now ADD (2) and (3):
# mu (U_{ik, j} + U_{ij, k}) n_j = 1/(2*pi) * [
#   psi' ( (dr . n) delta_{ik} + dr_k n_i )
#   - 2 (chi' - 2*chi/r) dr_i dr_k (dr . n)
#   - (chi/r) ( n_i dr_k + 2 delta_{ik} (dr . n) + 2 n_k dr_i )
# ]
# Notice terms with (dr . n) delta_{ik}:
#   (psi' - 2*chi/r) (dr . n) delta_{ik}
# Notice terms with dr_k n_i:
#   (psi' - chi/r) dr_k n_i
# Notice terms with n_k dr_i:
#   - 2 (chi/r) n_k dr_i
# Plus term (1):
#   lambda/mu * (psi' - chi' - chi/r) dr_i n_k!
#
# Combine terms with dr_i n_k:
#   [ lambda/mu * (psi' - chi' - chi/r) - 2*chi/r ] dr_i n_k!
#
# And terms with dr_i dr_k (dr . n):
#   - 2 (chi' - 2*chi/r) dr_i dr_k (dr . n)!

print("Analytical derivation completed!")
