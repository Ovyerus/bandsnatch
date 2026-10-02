# Bandcamp collection page JSON

This is a field map of the JSON in the `data-blob` attribute of the element with
`id="pagedata"` on a logged-in fan's `https://bandcamp.com/<username>` page. It
was checked against one full `debug_collection.json` dump; it is **not** a
Bandcamp API contract. Fields and types may differ across accounts or change
without notice. Paths below are relative to the blob root; `*` means a dynamic
object key or an array element. `null` means null in this particular dump, not
necessarily always null.

The local dump is ignored by Git. **Do not commit or paste a full dump:** it can
contain account/profile details, purchase history, signed action query strings,
and download links. `bandsnatch debug-collection --full --save` still replaces
`collection_data.redownload_urls` and the collection/hidden `sequence` and
`pending_sequence` values with `"[redacted by bandsnatch]"`. The inspected file
had those positions as objects/arrays; check for the redaction string before
using a debug-command dump as input. Without `--full`, the command emits only
`collection_data`, `fan_data`, and `hidden_data`.

## Where the downloader reads

| Path                    | Layout and role                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `fan_data`              | Profile of the page owner. `fan_id` (JSON number; deserialized as a string), `username`, `name`, `location`, `raw_location`, `bio`, `website_url`, `trackpipe_url` (strings); `photo` (`image_id`, `width`, `height` numbers); `is_own_page` (boolean); `followers_count`, `following_bands_count`, `following_fans_count`, `following_genres_count`, `subscriptions_count` (numbers); `fav_genre` (string). The downloader requires `is_own_page == true`. |
| `collection_data`       | Visible collection pagination and downloads; fields detailed below.                                                                                                                                                                                                                                                                                                                                                                                         |
| `hidden_data`           | Hidden collection pagination; fields detailed below. Current downloader sets `skip_hidden_items = true`.                                                                                                                                                                                                                                                                                                                                                    |
| `item_cache.collection` | Object keyed by sale-item key (e.g. `p<digits>`; this dump also has `a<digits>` and `t<digits>`). Values are release records; details below. The downloader filters download URLs against these records.                                                                                                                                                                                                                                                    |

The downloader deserializes only `fan_data`, `collection_data`, `hidden_data`,
and `item_cache.collection` (`src/api/structs/mod.rs`). Serde ignores the other
root fields. `src/api/mod.rs::get_download_urls` takes
`collection_data.redownload_urls`, joins its keys to records by
`sale_item_type + sale_item_id`, and paginates `collection_items` when
`item_count > batch_size`. The pagination response is a **different** object:
`{ items: Item[], redownload_urls: object, last_token: string, more_available: boolean }`;
the page blob does not contain all subsequent pages. Download URLs themselves
are not the release metadata records.

### Collection and pagination fields

| Object                                                          | Fields in this dump                                                                                                                                                                                                                                                                                                                                                         |
| --------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `collection_data`                                               | `redownload_urls` (map of sale-item key to download URL in the live blob; redacted by the debug command), `last_token` (string), `item_count`, `batch_size`, `hidden_items_count` (numbers), `small_collection`, `small_wishlist` (booleans), `purchase_infos`, `collectors` (objects, empty here), `sequence`, `pending_sequence` (arrays, redacted by the debug command). |
| `hidden_data`                                                   | `last_token` (string), `last_token_is_gift_given` (boolean), `item_count`, `batch_size` (numbers), `sequence`, `pending_sequence` (arrays, redacted).                                                                                                                                                                                                                       |
| `wishlist_data`                                                 | `last_token` (string), `item_count`, `batch_size` (numbers), `hidden` (boolean), `sequence`, `pending_sequence` (arrays).                                                                                                                                                                                                                                                   |
| `gifts_given_data`                                              | `visible_count`, `hidden_count`, `item_count`, `batch_size` (numbers), `last_token` (null here), `similar` (empty object here), `sequence`, `pending_sequence` (arrays).                                                                                                                                                                                                    |
| `followers_data`, `following_bands_data`, `following_fans_data` | Each has `last_token` (string or null here), `batch_size`, `item_count` (numbers), `sequence`, `pending_sequence` (arrays).                                                                                                                                                                                                                                                 |
| `following_genres_data`                                         | Same pagination fields, plus `hidden` (boolean).                                                                                                                                                                                                                                                                                                                            |
| `fan_suggestions_data`                                          | `item_count` (number), `sequence`, `pending_sequence` (arrays).                                                                                                                                                                                                                                                                                                             |

`item_count` can exceed the number of records initially present in
`item_cache.collection` or `collection_data.redownload_urls`; use the pagination
token rather than assuming the page contains the whole collection. Populated
`sequence` and `pending_sequence` arrays in this raw capture contain string IDs;
the debug command masks collection/hidden arrays. The Rust model permits
absent/null `batch_size`, `item_count`, `last_token`, and `redownload_urls`.

### Release records and tracklists

`item_cache` has maps named `collection`, `wishlist`, `gifts_given`, `hidden`,
`followers`, `following_bands`, `following_fans`, `following_genres`, and
`fan_suggestions`. In this dump, collection/wishlist/hidden entries share the
following record layout:

| Field group      | Fields and observed types                                                                                                                                                                                                                                                                                                                                                                                                    |
| ---------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Identity         | `sale_item_id`, `item_id`, `tralbum_id`, `band_id` (numbers); `sale_item_type`, `item_type`, `tralbum_type` (strings); `album_id` (number/null). The sample has `sale_item_type: "p"` even for records whose `item_type` is `album`, `package`, or `track`: do not substitute `item_type` when forming the download-map key.                                                                                                 |
| Display and URLs | `band_name`, `item_title`, `item_url`, `purchased` (strings); `band_location` (null here); `band_image_id` (null here); `item_art_id`, `featured_track`, `also_collected_count`, `num_streamable_tracks` (numbers); `featured_track_title` (string), `featured_track_url` (string/null).                                                                                                                                     |
| State            | `download_available`, `featured_track_is_custom`, `is_giftable`, `is_preorder`, `is_private`, `is_purchasable`, `is_subscriber_only`, `is_subscription_item` (booleans); `hidden`, `message_count`, `release_count`, `gift_id`, `gift_recipient_name`, `gift_sender_name`, `gift_sender_note`, `service_name`, `service_url_fragment`, `why` (null in this dump).                                                            |
| Nested           | `url_hints` (object with `subdomain`, `slug`, `item_type` strings, `custom_domain`, `custom_domain_verified` null here); `package_details` (object/null). When present, package details include `title`, `description`, `band_id`, `label_id`, `band_name`, `item_url`, `url_hints`, `images` (array of `{image_id, width, height}` numbers), `is_subscriber_only`, `is_live_event`, `private`, `killed`, `live_event_type`. |

`item_cache.followers` and `item_cache.following_bands` are separate kinds of
records keyed by numeric ID strings, not music releases. Their observed fields
are respectively `band_id`, `date_followed`, `fan_id`, `fan_url`, `image_id`,
`is_following`, `location`, `name`, `token`, `trackpipe_url`; and `art_id`,
`band_id`, `date_followed`, `image_id`, `is_following`, `is_subscribed`,
`location`, `name`, `token`, `url_hints`. Empty maps in this dump do not
establish shapes for the remaining item-cache categories.

`tracklists` has `collection`, `wishlist`, `gifts_given`, and `hidden` maps.
Collection/wishlist/hidden keys use the same kinds of sale-item keys as their
item caches; values are arrays of tracks. Track objects have `id` (number),
`title`, `artist` (strings), `track_number` (number/null), `duration` (number),
and `file` (object; the observed entries have an `mp3-v0` URL string). These
track URLs are distinct from the `redownload_urls` download map.

## Other page sections

These fields appeared at the root of this dump; they are **not** consumed by the
current downloader. Grouping is for navigation, not a promise that their values
are stable.

| Section                  | Fields / layout                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Signed actions and flags | `signup_querystrings` (map of item keys to signed query strings), `cfg` (object of site feature booleans), `templglobals` (`endpoint_mobilized`, `is_phone` booleans), `signup_params` (`from` string), `fan_onboarding` (onboarding flags, counts and nullable tooltip/profile fields), `social_prefs` (`fb_is_connected`, `tw_is_connected` booleans).                                                                                                                                                                                                                      |
| Identity                 | `identities` (`user` with numeric `id`; `fan` with `id`, `username`, `name`, `photo`, `private`, `verified`, `url`; `bands` array; `labels` array; `ip_country_code`, `partner`, `is_admin`, `is_page_band_member`, `subscribed_to_page_band`). `current_fan` (`fan_id`, `username`, `trackpipe_url`, `collection_count`, `subscriptions_count`, `is_following`, `is_following_any`, `is_own_collection`, `item_lookup`); `item_lookup` maps numeric ID strings to `{item_type: string, purchased: boolean}`. `fan_stats` holds `fan_id` and numeric activity/visit counters. |
| Locale and currency      | `languages` (language-code-to-name map), `currency_data` (`info` map of currency descriptors, `list` array of currency codes, `rates` map of numbers, `setting`, `current`); `genre_picker` (`genres` array of `{id, name, norm_name, value}`, `subgenres` map of arrays of `{name, norm_name, value}`).                                                                                                                                                                                                                                                                      |
| Presentation             | `banner_data` (banner choices array and current/custom banner IDs, URLs, alignments, image hash, `is_custom_banner`); `embed_data` (`linkback` string); `fan_suggestions_data` (pagination fields above).                                                                                                                                                                                                                                                                                                                                                                     |
| Root scalar settings     | `recaptcha_public_key`, `invisible_recaptcha_public_key`, `locale`, `help_center_url`, `utc_for_new_banner`, `platform`, `active_tab`, `mobile_app_url` (strings); `localize_page`, `image_editor_enabled`, `media_mode_test`, `fan_read_only`, `mobile_app_compatible`, `REVIEW_OR_FAVTRACK_FOUND` (booleans); `collection_count`, `MAX_NAME_LENGTH`, `MAX_BIO_LENGTH`, `MAX_WEBSITE_URL_LENGTH`, `MAX_WHY_LENGTH` (numbers); `from_fansuggest`, `platform_app_url`, `show_newsletter_invite_banner` (null here).                                                            |

For a new capture, compare key presence and types before relying on this map:

```sh
jq -r 'to_entries[] | "\(.key)\t\(.value | type)"' debug_collection.json
jq -r '.item_cache.collection | [.[] | to_entries[] | "\(.key): \(.value | type)"] | unique[]' debug_collection.json
```

These commands print field names/types, not private field values.
