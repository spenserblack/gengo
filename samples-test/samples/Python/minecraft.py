from uuid import uuid4
from unittest import TestCase

class Item:
    def __init__(self):
        self.id = uuid4()

class Charcoal(Item):
    pass

class Stick(Item):
    def __add__(self, other):
        return Torch(self, other)

class Torch(Item):
    def __init__(self, stick: Stick, charcoal: Charcoal):
        super().__init__()
        self.stick = stick
        self.charcoal = charcoal

    def components(self):
        return [self.stick, self.charcoal]

class TestMinecraft(TestCase):
    def test_torch(self):
        stick = Stick()
        charcoal = Charcoal()
        torch = stick + charcoal
        components = torch.components()

        self.assertIn(charcoal, components)
        self.assertIn(stick, components)
