shader_type spatial;
render_mode unshaded;

uniform sampler2D layer0 : hint_default_white;
uniform sampler2D layer1 : hint_default_white;
uniform sampler2D layer2 : hint_default_white;
uniform sampler2D layer3 : hint_default_white;
uniform int filter0;
uniform int filter1;
uniform int filter2;
uniform int filter3;
uniform int layer_count;

vec4 apply_filter(vec4 base, vec4 next, int filter_mode) {
	// FilterMode::None_ = 0
	if (filter_mode == 0) {
		return next;
	}
	// FilterMode::Transparent = 1
	if (filter_mode == 1) {
		if (next.a < 0.75) {
			discard;
		}
		return next;
	}
	// FilterMode::Blend = 2
	if (filter_mode == 2) {
		return vec4(mix(base.rgb, next.rgb, next.a), base.a);
	}
	// FilterMode::Additive = 3
	if (filter_mode == 3) {
		return vec4(base.rgb + next.rgb, base.a);
	}
	// FilterMode::AddAlpha = 4
	if (filter_mode == 4) {
		return vec4(base.rgb + next.rgb * next.a, base.a);
	}
	// FilterMode::Modulate = 5
	if (filter_mode == 5) {
		return vec4(base.rgb * next.rgb, base.a);
	}
	// FilterMode::Modulate2x = 6
	if (filter_mode == 6) {
		return vec4(base.rgb * next.rgb * 2.0, base.a);
	}
	return base;
}

void fragment() {
	vec4 c = texture(layer0, UV);

	if (layer_count > 1) {
		vec4 next = texture(layer1, UV);
		c = apply_filter(c, next, filter1);
	}
	if (layer_count > 2) {
		vec4 next = texture(layer2, UV);
		c = apply_filter(c, next, filter2);
	}
	if (layer_count > 3) {
		vec4 next = texture(layer3, UV);
		c = apply_filter(c, next, filter3);
	}

	ALBEDO = c.rgb;
	ALPHA = c.a;
}
