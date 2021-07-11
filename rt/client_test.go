package main

import "testing"

func roomIdsEQ(a []RoomId, b []RoomId) bool {
	if len(a) != len(b) {
		return false
	}
	for i := range a {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}

func roomIdsNE(a []RoomId, b []RoomId) bool {
	return !roomIdsEQ(a, b)
}

func TestRemoveRoomId(t *testing.T) {
	arr := []RoomId{
		{Q: 1, R: 1},
		{Q: 2, R: 4},
		{Q: 1, R: 4},
	}

	res := RemoveRoomId(arr, RoomId{Q: 2, R: 4})

	if roomIdsEQ(arr, res) {
		t.Error("No items were removed")
	}

	if roomIdsNE(res, []RoomId{
		{Q: 1, R: 1},
		{Q: 1, R: 4},
	}) {
		t.Errorf("Result array mismatch: %v", res)
	}
}

func TestRemoveRoomIdNotInArrayDoesNothing(t *testing.T) {
	arr := []RoomId{
		{Q: 1, R: 1},
		{Q: 2, R: 4},
		{Q: 1, R: 4},
	}

	res := RemoveRoomId(arr, RoomId{Q: 2, R: -10})

	if !roomIdsEQ(arr, res) {
		t.Error("Array was modified, expected the same array")
	}
}
