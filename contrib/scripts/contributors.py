import subprocess
import re
import sys


def get_contributors(commit_range=None, since=None, until=None):
    """
    Extract authors and co-authors based on a commit range or optional date filters.

    :param commit_range: Git range (e.g., v1.0.0..v2.0.0, main..feature-branch, or None).
    :param since: Date to filter commits starting from (e.g., '2023-01-01').
    :param until: Date to filter commits up to (e.g., '2023-12-31').
    :return: Sorted list of unique contributors.
    """

    contributors = set()

    # Base git log command
    log_command = ["git", "log", "--pretty=format:%an <%ae>"]

    # Add commit range if provided
    if commit_range:
        log_command.insert(2, commit_range)

    # Add date filters if provided
    if since:
        log_command.append(f"--since={since}")
    if until:
        log_command.append(f"--until={until}")

    # Step 1: Get main authors
    result = subprocess.run(
        log_command,
        capture_output=True,
        text=True,
        check=True,
    )
    for author in result.stdout.splitlines():
        contributors.add(author)

    # Step 2: Extract commits with co-authors
    # Only include commits containing "Co-authored-by:" in the message
    co_author_command = log_command.copy()
    co_author_command.append("--grep=Co-authored-by:")
    co_author_command[-1] = "--pretty=format:%H"

    # Get commit hashes
    result = subprocess.run(
        co_author_command,
        capture_output=True,
        text=True,
        check=True,
    )
    commit_hashes = result.stdout.splitlines()

    # Get co-authors by inspecting commit messages
    for commit in commit_hashes:
        result = subprocess.run(
            ["git", "show", "--quiet", commit],
            capture_output=True,
            text=True,
            check=True,
        )
        commit_message = result.stdout
        coauthors = re.findall(r"Co-authored-by: (.+ <.+>)", commit_message)
        contributors.update(coauthors)

    return sorted(contributors)


if __name__ == "__main__":
    # Command line parsing: supports ranges and optional dates
    import argparse

    parser = argparse.ArgumentParser(
        description="Analyze contributors (authors and co-authors) for a range of commits or dates."
    )
    parser.add_argument(
        "--range",
        type=str,
        help="Specify a commit range (e.g., 'v1.0.0..v2.0.0' or 'main..feature-branch').",
    )
    parser.add_argument(
        "--since", type=str, help="Start date for filtering commits (e.g., '2023-01-01')."
    )
    parser.add_argument(
        "--until", type=str, help="End date for filtering commits (e.g., '2023-12-31')."
    )

    args = parser.parse_args()

    # Input validation
    if not args.range and not (args.since or args.until):
        print("Error: You must specify either '--range' or at least one of '--since'/'--until'.")
        parser.print_help()
        sys.exit(1)

    contributors = get_contributors(commit_range=args.range, since=args.since, until=args.until)

    if contributors:
        print("Contributors:")
        print("-" * 40)
        for contributor in contributors:
            print(contributor)
    else:
        print("No contributors found for the specified filters.")
