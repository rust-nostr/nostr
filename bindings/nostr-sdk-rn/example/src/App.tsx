import { Text, View, StyleSheet } from "react-native";
import { Keys } from "../../src";

const keys = Keys.generate();

export default function App() {
	return (
		<View style={styles.container}>
			<Text>Keys: {JSON.stringify(keys)}</Text>
		</View>
	);
}

const styles = StyleSheet.create({
	container: {
		flex: 1,
		alignItems: "center",
		justifyContent: "center",
	},
});
