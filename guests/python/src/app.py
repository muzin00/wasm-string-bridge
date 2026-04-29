import wit_world


class WitWorld(wit_world.WitWorld):
    def process_string(self, input: str) -> str:
        return "".join(
            chr(ord(c) - 32) if "a" <= c <= "z" else c for c in input
        )
