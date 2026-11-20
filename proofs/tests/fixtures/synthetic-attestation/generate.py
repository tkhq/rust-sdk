"""Regenerate controlled P-384 test roots. These certificates are not AWS evidence."""
from datetime import datetime, timezone
from pathlib import Path
from cryptography import x509
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.x509.oid import NameOID, ExtendedKeyUsageOID

out = Path(__file__).parent
root_key = ec.derive_private_key(int.from_bytes(bytes([41]) * 48, 'big'), ec.SECP384R1())
leaf_key = ec.derive_private_key(int.from_bytes(bytes([42]) * 48, 'big'), ec.SECP384R1())
root_name = x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, 'TVC synthetic test root')])
leaf_name = x509.Name([x509.NameAttribute(NameOID.COMMON_NAME, 'TVC synthetic attestation')])
start = datetime(2024, 1, 1, tzinfo=timezone.utc)
end = datetime(2025, 1, 1, tzinfo=timezone.utc)
root = (x509.CertificateBuilder().subject_name(root_name).issuer_name(root_name)
        .public_key(root_key.public_key()).serial_number(1).not_valid_before(start)
        .not_valid_after(datetime(2035, 1, 1, tzinfo=timezone.utc))
        .add_extension(x509.BasicConstraints(ca=True, path_length=1), critical=True)
        .add_extension(x509.KeyUsage(False, False, False, False, False, True, True, None, None), critical=True)
        .sign(root_key, hashes.SHA384()))
leaf = (x509.CertificateBuilder().subject_name(leaf_name).issuer_name(root_name)
        .public_key(leaf_key.public_key()).serial_number(2).not_valid_before(start).not_valid_after(end)
        .add_extension(x509.BasicConstraints(ca=False, path_length=None), critical=True)
        .add_extension(x509.KeyUsage(True, False, False, False, False, False, False, None, None), critical=True)
        .add_extension(x509.ExtendedKeyUsage([ExtendedKeyUsageOID.SERVER_AUTH]), critical=False)
        .sign(root_key, hashes.SHA384()))
for name, cert in [('root.der', root), ('leaf.der', leaf)]:
    (out / name).write_bytes(cert.public_bytes(serialization.Encoding.DER))
